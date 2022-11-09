use crate::bridge::{LazyCell,SyncOnceCell};
use crate::bridge::graphics::{Context,ANativeWindow};
use crate::bridge::activity::Activity;
use jni::{JNIEnv,JavaVM};
use jni::objects::GlobalRef;
use std::path::PathBuf;
use std::sync::{RwLock,Mutex,Condvar,Arc};
use std::ptr::{null, null_mut};
use vulkano::VulkanLibrary;
use vulkano::instance::Instance;
use std::ops::Deref;

// Activities can be created and can be destroyed at any time,
// so they must be optional. However, they're required for
// many things on Android, so protect them with an RwLock + Condvar.
// Luckily, the writes we do are very uncommon and terse. 
pub struct AndroidFFI {
    pub activity: Option<Activity>,
    pub context: Option<Context>,
}
pub static ANDROID_FFI_MUTEX: RwLock<AndroidFFI> = RwLock::new(
  AndroidFFI { activity: None, context: None },
);
pub static ANDROID_FFI_CONDVAR: Condvar = Condvar::new();

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

// Cache Vulkan library
pub static VULKAN_LIBRARY: LazyCell<Arc<VulkanLibrary>> = LazyCell::new(|| {
    return VulkanLibrary::new().expect("Unable to load Vulkan library!!!");
});

pub static VULKAN_INSTANCE: LazyCell<Arc<Instance>> = LazyCell::new(|| {
    return Instance::new(
        VULKAN_LIBRARY.deref().clone(),
        Default::default(),
    ).expect("Unable to create Vulkan instance!!!");
});

