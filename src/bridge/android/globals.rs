use crate::bridge::{LazyCell,SyncOnceCell};
use jni::{JNIEnv,JavaVM};
use jni::objects::GlobalRef;
use std::path::PathBuf;

// We need a global AssetManager to get Android assets.
// We can get a global reference and forget about it.
pub static ASSET_MANAGER: SyncOnceCell<GlobalRef> = SyncOnceCell::new();

// All the path roots we care about in our game.
// These are initialized if bridgeOnCreate finishes.
pub static INTERNAL_PATH: SyncOnceCell<PathBuf> = SyncOnceCell::new();

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
}

