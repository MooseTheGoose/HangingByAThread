use std::os::raw::c_int;

use std::os::unix::io::AsRawFd;
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

use jni::{JNIEnv, JavaVM};
use jni::objects::*;
use jni::sys::*;

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

// Cache the Java VM so all threads can access it.
static mut JVM: Option<JavaVM> = None;

// Make JNI a thread-local variable, since the
// cost of TLS is very small compared to the cost
// of JNI and makes this global thread-safe.
thread_local! {
  static JNI: RefCell<Option<JNIEnv<'static>>> = RefCell::new(None);
}

fn get_global_jni<'a>(opt: Option<&'a JNIEnv<'a>>) -> Result<&'a JNIEnv<'a>> {
    return match opt {
        Some(r) => Ok(r),
        None => Err(Error::JNINotInitialized),
    };
}

// Error message to print when JNI not found
static JNI_NOT_FOUND_MSG: &'static str = "JNI not initialized in this thread!";

// We need a global AssetManager to get Android assets.
// We can get a global reference and forget about it.
static mut ASSET_MANAGER: MaybeUninit<GlobalRef> = MaybeUninit::uninit();

// All the path roots we care about in our game.
// These are initialized if bridgeOnCreate finishes.
static mut INTERNAL_PATH: MaybeUninit<PathBuf> = MaybeUninit::uninit();

pub fn internal_path() -> &'static Path {
    return unsafe { INTERNAL_PATH.assume_init_ref().as_ref() }
}

struct Asset {
    istream: GlobalRef,
}

impl Asset {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Asset> {
        return JNI.with(|jnicell| -> Result<Asset> {
            let jniopt = jnicell.borrow();
            let jniref = get_global_jni(jniopt.as_ref())?;
            let res = {
                let mgrglobal = unsafe { ASSET_MANAGER.assume_init_read() };
                let mgr = mgrglobal.as_obj();
                let fname_str = match path.as_ref().to_str() {
                    Some(s) => Ok(s),
                    None => Err(Error::UTF8DecodeError),
                }?;
                let jstr = jniref.new_string(fname_str)?;
                let fname_obj = JValue::Object(JObject::from(jstr));
                match jniref.call_method(mgr, "open", "(Ljava/lang/String;)Ljava/io/InputStream;", &[fname_obj]) {
                    Ok(r) => Ok(jniref.new_global_ref(r.l()?)?),
                    Err(e) => Err(Error::JNIError(e)),
                }
            };
            let _ =  {
                if jniref.exception_check()? {
                    jniref.exception_clear()?;
                }
            };
            Ok(Asset { istream: res? })
        });
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
        let jniopt = jnicell.borrow();
        let jniref = get_global_jni(jniopt.as_ref())?;
        let files_dir = jniref.call_method(ctx, "getFilesDir", "()Ljava/io/File;", &[])?.l()?;
        let abs_path = JString::from(jniref.call_method(files_dir, "getAbsolutePath", "()Ljava/lang/String;", &[])?.l()?);
        let buf = PathBuf::from(
            jniref.get_string(abs_path)?.to_str()?
        );
        unsafe { INTERNAL_PATH.write(buf); }
        Ok(())
    }).expect("Failed to get file roots from JNI");
}

fn init_asset_manager(ctx: JObject) {
    JNI.with(|jnicell| -> Result<()> {
        let jniopt = jnicell.borrow();
        let jniref = get_global_jni(jniopt.as_ref())?;
        let asset_manager = jniref.call_method(ctx, "getAssets", "()Landroid/content/res/AssetManager;", &[])?.l()?;
        let global_manager = jniref.new_global_ref(asset_manager)?;
        unsafe { ASSET_MANAGER.write(global_manager); }
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
        unsafe { JVM = Some(env.get_java_vm().expect("Unable to get JVM from JNI!")); }
        initialize_thread_runtime();
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
        initialize_thread_runtime();
        let asset = Asset::open("models-le/SimpleBox.model");
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

// Initialize the runtime for our thread
// If you forget it, you might panic because
// JNI isn't initialized. You can't reasonably
// continue if you fail this, so just panic.
pub fn initialize_thread_runtime() {
    JNI.with(|opt| {
        let mut jniref = opt.borrow_mut();
        if (*jniref).is_some() { return; }
        unsafe { 
            let jvm = JVM.as_ref().expect("WTF?! JVM not initialized?");
            *jniref = Some(jvm.attach_current_thread_as_daemon()
                      .expect("Failed to read JNI from JVM"));
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


