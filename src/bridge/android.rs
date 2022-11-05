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
use android_logger::Config;

use jni::{JNIEnv, JavaVM};
use jni::objects::{JObject, JString};
use jni::sys::*;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    JNIError(jni::errors::Error),
}

// Cache the Java VM so all threads can access it.
static mut JVM: Option<JavaVM> = None;

// Make JNI a thread-local variable, since the
// cost of TLS is very small compared to the cost
// of JNI and makes this global thread-safe.
thread_local!(static JNI: RefCell<Option<JNIEnv<'static>>> = RefCell::new(None));

// All the path roots we care about in our game.
// These are initialized if bridgeOnCreate finishes.
static mut INTERNAL_PATH: MaybeUninit<PathBuf> = MaybeUninit::uninit();

pub fn internal_path() -> &'static Path {
    return unsafe { INTERNAL_PATH.assume_init_ref().as_ref() }
}

// I must use some function in this bridge
// in the C++ file for Android Studio. Otherwise,
// this static library gets optimized out.
#[no_mangle]
pub extern "C" fn hbat_bridge_stub() {
}

fn init_internal_path(ctx: JObject) -> PathBuf {
    return JNI.with(|jnicell| -> jni::errors::Result<PathBuf> {
        let jniopt = jnicell.borrow();
        let jniref = *jniopt.as_ref().expect("JNI not initialized in this thread!");
        let files_dir = jniref.call_method(ctx, "getFilesDir", "()Ljava/io/File;", &[])?.l()?;
        let abs_path  = JString::from(jniref.call_method(files_dir, "getAbsolutePath", "()Ljava/lang/String;", &[])?.l()?);
        let buf = PathBuf::from(
            jniref.get_string(abs_path)?.to_str().expect("WTF!?!? Internal path is not UTF-8?")
        );
        Ok(buf)
    }).expect("Failed to get internal path from JNI");
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
        unsafe {
            INTERNAL_PATH.write(init_internal_path(ctx));
        }
        spawn(|| { devcon_loop(); });
    }).is_ok() {
        abort();
    }
}

// Called when EGL context is lost.
// Recreate resources associated with last CTX
#[no_mangle]
pub extern "system" fn Java_com_binaryquackers_hbat_MainGLSurfaceView_00024MainGLRenderer_bridgeOnSurfaceCreated(
    _env: JNIEnv, _renderer: JObject, _ctx: JObject) {
    let _ = catch_unwind(|| {
        initialize_thread_runtime();
        println!("Hello, World!");
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


