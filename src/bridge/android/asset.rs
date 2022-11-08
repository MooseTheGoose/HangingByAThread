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
            return unsafe { 
                let res = MmapOptions::new().offset(start as u64).len(len).map(fd);
                close(fd);
                Ok(res?)
            };
        });
    }

    pub fn map<P: AsRef<Path>>(path: P) -> Result<Mmap> {
        let res = Self::map_helper(path);
        describe_and_clear_jni_exception();
        return res;
    }
}
