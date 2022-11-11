use jni::objects::{GlobalRef,JObject,JString,JValue,JClass};
use crate::bridge::{Result,Error};
use jni::JNIEnv;
use jni::descriptors::Desc;
use std::path::PathBuf;
use std::io::{BufRead,Read};
use std::vec::Vec;
use std::path::Path;
use std::ops::Deref;
use crate::bridge::globals::*;
use memmap2::*;
use libc::close;

// Wrapper around JNIEnv that, when
// dropped, will check exceptions and
// clear them.
pub struct JNICatch<'a>(pub &'a JNIEnv<'a>);

impl Drop for JNICatch<'_> {
    fn drop(&mut self) {
        match self.0.exception_check() {
            Ok(chk) => {
                if chk {
                    match self.0.exception_describe() {
                        Err(e) => log::error!("JNI Exception Description failed: {:?}", e),
                        _ => {},
                    }
                    match self.0.exception_clear() {
                        Err(e) => log::error!("JNI Exception Clear failed: {:?}", e),
                        _ => {},
                    }
                } 
            },
            Err(e) => log::error!("JNI Exception Check failed: {:?}", e),
        };
    }
}

pub struct Activity {
    g_ref: GlobalRef,
    asset_manager: GlobalRef,
    internal_path: PathBuf,
}

impl Activity {
    pub fn new(env: JNIEnv, activity: JObject) -> Result<Activity> {
        // Asset Manager
        let activity_ref = env.new_global_ref(activity)?;
        let asset_manager = env.call_method(activity, "getAssets",
            "()Landroid/content/res/AssetManager;", &[])?.l()?;
        let global_manager = env.new_global_ref(asset_manager)?;
        // Paths
        let files_dir = env.call_method(activity, "getFilesDir",
            "()Ljava/io/File;", &[])?.l()?; 
        let files_string = JString::from(env.call_method(files_dir, "getAbsolutePath", "()Ljava/lang/String;", &[])?.l()?);
        let internal_buf = PathBuf::from(env.get_string(files_string)?.to_str()?);
        // Return the final activity.
        return Ok(Activity {
            g_ref: activity_ref,
            asset_manager: global_manager,
            internal_path: internal_buf,
        });
    }
}

// Read from Assets folder on Android.
pub struct Asset<'a> {
    channel: GlobalRef,
    bytebuffer: GlobalRef,
    vecbuffer: Vec<u8>,
    input_data: &'a [u8]
}

impl Asset<'_> {
    pub fn open<P: AsRef<Path>>(path: P, activity: &Activity) -> Result<Asset> {
        return JNI.with(|jnicell| -> Result<Asset> {
            let jniref = jnicell.deref();
            let _catcher = JNICatch(jniref);
            let channel = {
                let mgrglobal = &activity.asset_manager;
                let mgr = mgrglobal.as_obj();
                let fname_str = match path.as_ref().to_str() {
                    Some(s) => Ok(s),
                    None => Err(Error::UTF8DecodeError),
                }?;
                let jstr = jniref.new_string(fname_str)?;
                let fname_obj = JValue::Object(JObject::from(jstr));
                let istream = jniref.call_method(mgr, "open", "(Ljava/lang/String;)Ljava/io/InputStream;", &[fname_obj])?.l()?;
                let channel_factory: JClass = "java/nio/channels/Channels".lookup(jniref)?;
                jniref.new_global_ref(jniref.call_static_method(channel_factory, "newChannel",
                    "(Ljava/io/InputStream;)Ljava/nio/channels/ReadableByteChannel;",
                    &[JValue::Object(istream)])?.l()?)
            }?;
            let mut data = Vec::with_capacity(0x2000);
            let len = data.capacity();
            // This bytebuffer becomes unsafe when you puhs anything in
            // the vector or when you pass this to Java code, so don't
            // do either of those.
            let (bytebuffer, slice) = unsafe {
                let data_ptr = data.as_mut_ptr();
                (jniref.new_global_ref(jniref.new_direct_byte_buffer(data_ptr, len)?)?,
                 std::slice::from_raw_parts(data_ptr, 0))
            };
            Ok(Asset { channel: channel, bytebuffer: bytebuffer, vecbuffer: data, input_data: slice})
        });
    }
}
impl BufRead for Asset<'_> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        if self.input_data.len() <= 0 {
            JNI.with(|jnicell| -> Result<()> {
                let jniref = jnicell.deref();
                let _catcher = JNICatch(jniref);
                let bytebuffer_obj = self.bytebuffer.as_obj();
                let channel_obj = self.channel.as_obj();
                jniref.call_method(bytebuffer_obj, "clear", "()Ljava/nio/Buffer;", &[])?.l()?;
                let mut amt_read = 0;
                while amt_read == 0 {
                    amt_read = jniref.call_method(channel_obj, "read", "(Ljava/nio/ByteBuffer;)I", &[JValue::Object(bytebuffer_obj)])?.i()?;
                }
                amt_read = std::cmp::max(amt_read, 0);
                self.input_data = unsafe { std::slice::from_raw_parts(self.input_data.as_ptr(), amt_read as usize) };
                Ok(())
            })?;
        }
        return Ok(self.input_data);
    }
    fn consume(&mut self, amt: usize) {
        let new_amt = std::cmp::min(amt, self.input_data.len());
        self.input_data = &self.input_data[new_amt..];
    }
}
impl Read for Asset<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let data = self.fill_buf()?;
        let amt = std::cmp::min(buf.len(), data.len());
        buf[..amt].clone_from_slice(&data[..amt]);
        self.consume(amt);
        return Ok(amt);
    }
}
impl Asset<'_> {
    pub fn map<P: AsRef<Path>>(path: P, activity: &Activity) -> Result<Mmap> {
        return JNI.with(|jnicell| -> Result<Mmap> {
            let jniref = jnicell.deref();
            let _catcher = JNICatch(jniref);
            let mgrglobal = &activity.asset_manager;
            let mgr = mgrglobal.as_obj();
            let fname_str = match path.as_ref().to_str() {
                Some(s) => Ok(s),
                None => Err(Error::UTF8DecodeError),
            }?;
            let jstr = jniref.new_string(fname_str)?;
            let fname_obj = JValue::Object(JObject::from(jstr));
            let afd_opt = jniref.call_method(mgr, "openFd", "(Ljava/lang/String;)Landroid/content/res/AssetFileDescriptor;", &[fname_obj]).ok();
            if afd_opt.is_none() {
                // Try again via creating an Asset instead of using openFd.
                // The exception must also be cleared, but don't report it.
                match jniref.exception_check() {
                    Ok(excp) => if excp { let _ = jniref.exception_clear(); },
                    _ => {},
                };
                let mut asset = Self::open(path, activity)?;
                let mut data = vec![];
                asset.read_to_end(&mut data)?;
                return {
                    let mut mmap_mut = MmapOptions::new().len(data.len()).map_anon()?;
                    let mmap_slice: &mut [u8] = &mut mmap_mut;
                    mmap_slice.clone_from_slice(&data[..]);
                    Ok(mmap_mut.make_read_only()?)
                };
            }
            let afd = afd_opt.unwrap().l()?;
            let start = jniref.call_method(afd, "getStartOffset", "()J", &[])?.j()?;
            let len: usize = match jniref.call_method(afd, "getLength", "()J", &[])?.j()?.try_into().ok() {
                Some(u) => Ok(u),
                None => Err(Error::NumericConversionError),
            }?;
            let pfd = jniref.call_method(afd, "getParcelFileDescriptor", "()Landroid/os/ParcelFileDescriptor;", &[])?.l()?;
            let fd = jniref.call_method(pfd, "detachFd", "()I", &[])?.i()?;
            return unsafe { 
                let res = MmapOptions::new().offset(start as u64).len(len).map(fd);
                close(fd);
                Ok(res?)
            };
        });
    }
}
