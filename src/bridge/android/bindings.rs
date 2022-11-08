use jni::{JavaVM,JNIEnv};
use jni::sys::{jboolean,JNI_FALSE,JNI_TRUE};
use jni::objects::*;
use crate::bridge::globals::*;
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

// Initialize the runtime here with this JNI method.
// Return if this initialization was successful or not.
#[no_mangle]
pub extern "system" fn Java_com_binaryquackers_hbat_MainActivity_bridgeOnCreate(
    env: JNIEnv, ctx: JObject) -> jboolean {
    // Set up first-time initialization here!
    return catch_unwind(|| -> jboolean{
        android_logger::init_once(
            Config::default().with_min_level(Level::Trace),
        );
        JVM.set(env.get_java_vm().expect("Unable to get JVM from JNI!")).unwrap();
        init_paths(ctx);
        init_asset_manager(ctx);
        // spawn(|| { devcon_loop(); });
        JNI_TRUE
    }).unwrap_or(JNI_FALSE);
}
