use std::os::raw::c_int;

use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixListener;
use libc::*;
use std::fs::*;
use std::ffi::{CString, CStr};
use std::ptr::{null};
use std::sync::Mutex;
use std::path::{PathBuf, Path};
use std::mem::MaybeUninit;
use crate::devcon::conmain;
use std::thread::*;
use std::panic::catch_unwind;
use std::process::abort;
use std::cell::*;
use log::Level;
use log::*;
use android_logger::Config;
use gl;
use memmap2::*;
use std::io::{Read};
use once_cell::sync::{OnceCell, Lazy};
use std::ops::Deref;

use jni::{JNIEnv, JavaVM};
use jni::objects::*;
use jni::sys::*;

type LazyCell<T, F = fn() -> T> = Lazy<T, F>;

#[link(name = "EGL")]
extern "C" {
    pub fn eglGetProcAddress(fname: *const c_char) -> *const c_void;
}
fn load_gl() {
    gl::load_with(|s| -> *const c_void {
        let cs = match CString::new(s) {
            Ok(tmp) => tmp,
            Err(_) => return null(),
        };
        let raw = cs.into_raw();
        let addr = unsafe { eglGetProcAddress(raw) };
        let _cs = unsafe { CString::from_raw(raw) };
        return addr;
    });
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    JNIError(jni::errors::Error),
    IOError(std::io::Error),
    RuntimeNotInitializedError,
    NumericConversionError,
    JNINotInitialized,
    UTF8DecodeError,
}

impl From<jni::errors::Error> for Error {
    fn from(e: jni::errors::Error) -> Self {
        return Error::JNIError(e);
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(_e: std::str::Utf8Error) -> Self {
        return Error::UTF8DecodeError;
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        return Error::IOError(e);
    }
}

// Cache the Java VM so all threads can access it.
static JVM: OnceCell<JavaVM> = OnceCell::new();

// Make JNI a thread-local variable, since the
// cost of TLS is very small compared to the cost
// of JNI and makes this global thread-safe.
thread_local! {
  static JNI: LazyCell<JNIEnv<'static>> = LazyCell::new(|| {
      let jvm = JVM.get().unwrap();
      return jvm.attach_current_thread_as_daemon()
                .expect("Failed to read JNI from JVM");
  });
}

fn describe_and_clear_jni_exception_helper() -> Result<()> {
    return JNI.with(|jnicell| -> Result<()> {
        let jniref = jnicell.deref();
        if jniref.exception_check()? {
            jniref.exception_describe()?;
            jniref.exception_clear()?;
        }
        Ok(())
    });
}

fn describe_and_clear_jni_exception() {
    let res = describe_and_clear_jni_exception_helper();
    if res.is_err() {
        warn!("Could not describe and clear JNI exception for some reason. Strange...");
    }
}

// We need a global AssetManager to get Android assets.
// We can get a global reference and forget about it.
static ASSET_MANAGER: OnceCell<GlobalRef> = OnceCell::new();

// All the path roots we care about in our game.
// These are initialized if bridgeOnCreate finishes.
static INTERNAL_PATH: OnceCell<PathBuf> = OnceCell::new();

pub fn internal_path() -> &'static Path {
    return INTERNAL_PATH.get().unwrap().as_ref();
}

struct Asset {
    istream: GlobalRef,
}

impl Asset {
    fn read_helper(&mut self, buf: &mut [u8]) -> Result<usize> {
        return JNI.with(|jnicell| -> Result<usize> {
            let jniref = jnicell.deref();
            let buflen: jsize = match buf.len().try_into().ok() {
                Some(l) => l,
                None => { return Err(Error::NumericConversionError); }
            };
            let bytearray = jniref.new_byte_array(buflen)?;
            let arrobj: JObject = unsafe { std::mem::transmute(bytearray) };
            let bytearray_val = JValue::Object(arrobj);
            let istream_obj = self.istream.as_obj();
            let nbytes_i32 = std::cmp::max(jniref.call_method(istream_obj, "read", "([B)I", &[bytearray_val])?.i()?, 0);
            Ok({
                let jnislice = unsafe { std::slice::from_raw_parts_mut(buf.as_ptr() as *mut jbyte, buf.len()) };
                let nbytes = nbytes_i32 as usize;
                jniref.get_byte_array_region(bytearray, 0, &mut jnislice[..nbytes])?;
                nbytes
            })
        });
    }
}

impl std::io::Read for Asset {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let res = self.read_helper(buf);
        describe_and_clear_jni_exception();
        return match res {
            Ok(r) => Ok(r),
            Err(e) => {
                match e {
                    Error::JNIError(_) => Err(std::io::Error::new(std::io::ErrorKind::Other, "JNI error occurred")),
                    Error::NumericConversionError => Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Read too big for JNI")),
                    _ => Err(std::io::Error::last_os_error()),
                }
            }
        };
    }
}

impl Asset {
    fn open_helper<P: AsRef<Path>>(path: P) -> Result<Asset> {
        return JNI.with(|jnicell| -> Result<Asset> {
            let jniref = jnicell.deref();
            let res = {
                let mgrglobal = ASSET_MANAGER.get().unwrap();
                let mgr = mgrglobal.as_obj();
                let fname_str = match path.as_ref().to_str() {
                    Some(s) => Ok(s),
                    None => Err(Error::UTF8DecodeError),
                }?;
                let jstr = jniref.new_string(fname_str)?;
                let fname_obj = JValue::Object(JObject::from(jstr));
                jniref.new_global_ref(jniref.call_method(mgr, "open", "(Ljava/lang/String;)Ljava/io/InputStream;", &[fname_obj])?.l()?)
            };
            Ok(Asset { istream: res? })
        });
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Asset> {
        let res = Self::open_helper(path);
        describe_and_clear_jni_exception();
        return res;
    } 

    fn map_helper<P: AsRef<Path>>(path: P) -> Result<Mmap> {
        return JNI.with(|jnicell| -> Result<Mmap> {
            let jniref = jnicell.deref();
            let mgrglobal = ASSET_MANAGER.get().unwrap();
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
                let mut asset = Self::open(path)?;
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
            unsafe { Ok(MmapOptions::new().offset(start as u64).len(len).map(fd)?) }
        });
    }

    pub fn map<P: AsRef<Path>>(path: P) -> Result<Mmap> {
        let res = Self::map_helper(path);
        describe_and_clear_jni_exception();
        return res;
    }
}

// I must use some function in this bridge
// in the C++ file for Android Studio. Otherwise,
// this static library gets optimized out.
#[no_mangle]
pub extern "C" fn hbat_bridge_stub() {
}

fn init_paths(ctx: JObject) {
    JNI.with(|jnicell| -> Result<()> {
        let jniref = jnicell.deref();
        let files_dir = jniref.call_method(ctx, "getFilesDir", "()Ljava/io/File;", &[])?.l()?;
        let abs_path = JString::from(jniref.call_method(files_dir, "getAbsolutePath", "()Ljava/lang/String;", &[])?.l()?);
        let buf = PathBuf::from(
            jniref.get_string(abs_path)?.to_str()?
        );
        INTERNAL_PATH.set(buf).unwrap();
        Ok(())
    }).expect("Failed to get file roots from JNI");
}

fn init_asset_manager(ctx: JObject) {
    JNI.with(|jnicell| -> Result<()> {
        let jniref = jnicell.deref();
        let asset_manager = jniref.call_method(ctx, "getAssets", "()Landroid/content/res/AssetManager;", &[])?.l()?;
        let global_manager = jniref.new_global_ref(asset_manager)?;
        ASSET_MANAGER.set(global_manager).unwrap();
        Ok(())
    }).expect("Failed to get AssetManager from JNI");
}

// Initialize the runtime here with this JNI method
// If we somehow fail to do this, don't even return
// back to Java. Just crash, burn, and die.
#[no_mangle]
pub extern "system" fn Java_com_binaryquackers_hbat_MainActivity_bridgeOnCreate(
    env: JNIEnv, ctx: JObject) {
    // Set up first-time initialization here!
    if !catch_unwind(|| {
        android_logger::init_once(
            Config::default().with_min_level(Level::Trace),
        );
        JVM.set(env.get_java_vm().expect("Unable to get JVM from JNI!")).unwrap();
        init_paths(ctx);
        init_asset_manager(ctx);
        load_gl();
        spawn(|| { devcon_loop(); });
    }).is_ok() {
        abort();
    }
}

// Called when EGL context is lost.
// Recreate resources associated with last CTX
#[no_mangle]
pub extern "system" fn Java_com_binaryquackers_hbat_MainGLSurfaceView_00024MainGLRenderer_bridgeOnSurfaceCreated(
    _env: JNIEnv, _renderer: JObject) {
    let _ = catch_unwind(|| {
        let asset = Asset::map("models-le/SimpleBox.model");
        match asset {
            Ok(_) => info!("SimpleBox opens up fine!"),
            Err(e) => error!("SimpleBox is not opening! {:?}", e),
        };
        info!("onSurfaceCreated");
    });
}

#[no_mangle]
pub extern "system" fn Java_com_binaryquackers_hbat_MainGLSurfaceView_00024MainGLRenderer_bridgeOnDrawFrame(
    _env: JNIEnv, _renderer: JObject) {
    let _  = catch_unwind(|| { 
        unsafe {
            gl::ClearColor(0.33, 0.6, 0.9, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            info!("onDrawFrame");
        }
    });
}

#[no_mangle]
pub extern "system" fn Java_com_binaryquackers_hbat_MainGLSurfaceView_00024MainGLRenderer_bridgeOnSurfaceChanged(
  _env: JNIEnv, _renderer: JObject, width: jint, height: jint) {
    let _ = catch_unwind(|| {
        unsafe {
            gl::Viewport(0, 0, width, height);
            // Also reinstantiate everything at this point.
            info!("onSurfaceChanged");
        }
    });
}

fn devcon_loop() {
    let listener_result = {
        let unixpath = internal_path().join("devcon");
        let _ = remove_file(unixpath.as_path()); 
        UnixListener::bind(unixpath.as_path())
    };
    match listener_result {
        Ok(listener) => {
            info!("Waiting for devcon clients...");
            for stream_result in listener.incoming() {
               match stream_result {
                   Ok(stream) => {
                       let fd = stream.as_raw_fd();
                       unsafe {
                           dup2(fd, 2);
                           dup2(fd, 1);
                           dup2(fd, 0);
                       }
                       conmain();
                   },
                   Err(_) => { 
                       warn!("Connection to devcon didn't succeed...");
                   }
               } 
            }
        },
        Err(e) => {
            error!("Unable to create UNIX socket for devcon! {:?}", e);
        }
    };
}


