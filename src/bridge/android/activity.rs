use jni::objects::{GlobalRef,JObject,JString};
use crate::bridge::{Result,Error};
use jni::JNIEnv;
use std::path::PathBuf;

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
