use jni::{JavaVM,JNIEnv};
use jni::sys::{jboolean,jint,JNI_FALSE,JNI_TRUE};
use jni::objects::*;
use crate::bridge::globals::*;
use crate::bridge::graphics::*;
use crate::bridge::activity::Activity;
use android_logger::*;
use crate::bridge::{Result,Error};
use std::ops::Deref;
use std::panic::catch_unwind;
use std::path::{PathBuf,Path};
use log::*;

pub fn describe_and_clear_jni_exception_helper() -> Result<()> {
    return JNI.with(|jnicell| -> Result<()> {
        let jniref = jnicell.deref();
        if jniref.exception_check()? {
            jniref.exception_describe()?;
            jniref.exception_clear()?;
        }
        Ok(())
    });
}

pub fn describe_and_clear_jni_exception() {
    let res = describe_and_clear_jni_exception_helper();
    if res.is_err() {
        warn!("Could not describe and clear JNI exception for some reason. Strange...");
    }
}

#[no_mangle]
extern "system" fn Java_com_binaryquackers_hbat_MainActivity_bridgeOnCreate(
  env: JNIEnv, activity: JObject) -> jboolean {
    return catch_unwind(|| -> jboolean {
        let jvm_res = JVM.set(env.get_java_vm().expect("Unable to get JVM from JNI!"));
        let activity = Activity::new(env, activity).expect("Unable to cache Activity!");
        let mut ffi = ANDROID_FFI_MUTEX.write().unwrap();
        ffi.activity = Some(activity);
        ANDROID_FFI_CONDVAR.notify_all();
        drop(ffi);
        // Initialize logging and start main thread if this is the first time.
        if jvm_res.is_ok() {
            android_logger::init_once(
                Config::default().with_min_level(Level::Trace),
            );
        }
        info!("Activity created!");
        JNI_TRUE
    }).unwrap_or(JNI_FALSE);
}

#[no_mangle]
extern "system" fn Java_com_binaryquackers_hbat_MainActivity_bridgeOnDestroy(
  _env: JNIEnv, _activity: JObject) -> jboolean {
    return catch_unwind(|| -> jboolean {
        let mut ffi = ANDROID_FFI_MUTEX.write().unwrap();
        ffi.activity = None;
        drop(ffi);
        info!("Activity destroyed!");
        JNI_TRUE 
    }).unwrap_or(JNI_FALSE);
}

#[no_mangle]
extern "system" fn Java_com_binaryquackers_hbat_MainActivity_00024MainSurfaceCallback_bridgeSurfaceChanged(
  env: JNIEnv, _callback: JObject, surface: JObject) -> jboolean {
    return catch_unwind(|| -> jboolean {
        let mut ffi = ANDROID_FFI_MUTEX.write().unwrap();
        ffi.context = Some(Context::new(env, surface)
                           .expect("Failed to recreate graphics context in MainSurfaceView.surfaceCreated"));
        ANDROID_FFI_CONDVAR.notify_all();
        drop(ffi);
        info!("Surface changed!");
        JNI_TRUE
    }).unwrap_or(JNI_FALSE);
}

#[no_mangle]
extern "system" fn Java_com_binaryquackers_hbat_MainActivity_00024MainSurfaceCallback_bridgeSurfaceDestroyed(
  _env: JNIEnv, _callback: JObject) -> jboolean {
    return catch_unwind(|| -> jboolean {
        let mut ffi = ANDROID_FFI_MUTEX.write().unwrap();
        ffi.context = None;
        drop(ffi);
        info!("Surface destroyed!");
        JNI_TRUE
    }).unwrap_or(JNI_FALSE);
}

