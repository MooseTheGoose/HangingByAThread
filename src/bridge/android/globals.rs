use crate::bridge::{LazyCell,SyncOnceCell};
use crate::bridge::graphics::{Graphics,ANativeWindow};
use crate::bridge::activity::Activity;
use jni::{JNIEnv,JavaVM};
use jni::objects::GlobalRef;
use std::path::PathBuf;
use std::sync::{RwLock,Mutex,Condvar,Arc};
use std::sync::mpsc::{Sender,Receiver};
use std::ptr::{null, null_mut};
use std::ops::Deref;
use raw_window_handle::HasRawWindowHandle;
use std::ffi::c_void;
use std::thread::ThreadId;


// Activities for Android FFI can be created and destroyed
// at any time, so we need to make it optional and locked.
// It's reasonable for multiple threads to use this, so
// hide it behind an RWLock and condvar.
pub static ACTIVITY_LOCK: RwLock<Option<Activity>> = RwLock::new(None);
pub static ACTIVITY_CONDVAR: Condvar = Condvar::new();

pub static GRAPHICS_MUTEX: Mutex<Option<Graphics>> = Mutex::new(None);
pub static GRAPHICS_CONDVAR: Condvar = Condvar::new();

// Cache the Java VM so all threads can access it.
pub static JVM: SyncOnceCell<JavaVM> = SyncOnceCell::new();

// Make JNI a thread-local variable, since the
// cost of TLS is very small compared to the cost
// of JNI and makes this global thread-safe.
thread_local! {
  pub static JNI: LazyCell<JNIEnv<'static>> = LazyCell::new(|| {
      let jvm = JVM.get().unwrap();
      return jvm.attach_current_thread_as_daemon()
                .expect("Failed to read JNI from JVM");
  });
  pub static LOCAL_THREAD_ID: SyncOnceCell<ThreadId> = SyncOnceCell::new();
}

pub static RENDERER_THREAD_ID: SyncOnceCell<ThreadId> = SyncOnceCell::new();
