use jni::{JavaVM,JNIEnv};
use jni::sys::{jboolean,jint,JNI_FALSE,JNI_TRUE};
use jni::objects::*;
use crate::bridge::globals::*;
use crate::bridge::graphics::*;
use crate::bridge::activity::{Activity,Asset};
use android_logger::*;
use crate::bridge::{Result,Error};
use std::ops::Deref;
use std::panic::catch_unwind;
use std::path::{PathBuf,Path};
use log::*;
use std::sync::atomic::{Ordering, fence};
use std::thread::{self,ThreadId};
use std::io::{Read,BufRead};


#[no_mangle]
extern "system" fn Java_com_binaryquackers_hbat_MainActivity_bridgeOnCreate(
  env: JNIEnv, activity: JObject) -> jboolean {
    return catch_unwind(|| -> jboolean {
        let jvm_res = JVM.set(env.get_java_vm().expect("Unable to get JVM from JNI!"));
        let activity = Activity::new(env, activity).expect("Unable to cache Activity!");
        let mut guard = ACTIVITY_LOCK.write().unwrap();
        *guard = Some(activity);
        ACTIVITY_CONDVAR.notify_all();
        drop(guard);
        // Initialize logging and start main thread if this is the first time.
        if jvm_res.is_ok() {
            android_logger::init_once(
                Config::default().with_min_level(Level::Trace),
            );
            thread::spawn(move || -> ! {
                let tid = thread::current().id();
                LOCAL_THREAD_ID.with(|tidcell| { 
                    tidcell.set(tid).expect("Local thread id already set?");
                });
                RENDERER_THREAD_ID.set(tid).expect("Renderer thread id already set?");
                loop {
                    let mut graphics = GRAPHICS_MUTEX.lock().unwrap();
                    while graphics.as_ref().is_none() {
                        graphics = GRAPHICS_CONDVAR.wait(graphics).unwrap();
                    }
                    let graphics_unwrapped = graphics.as_mut().unwrap();
                    crate::mainloop::render(graphics_unwrapped);
                    unsafe { graphics_unwrapped.swap_buffers(); }
                    drop(graphics);
                }
            });
        }
        let tmp = ACTIVITY_LOCK.read().unwrap();
        match Asset::open("android/shaders/basic/basic.vert", tmp.as_ref().unwrap()) {
            Ok(asset) => {
                info!("Opened asset! Reading it line by line");
                for line in asset.lines() {
                    match line {
                        Ok(l) => info!("{}", l),
                        Err(e) => error!("Failed to read a line! {:?}", e),
                    };
                }
            },
            Err(e) => {
                error!("Failed to open asset! {:?}", e);
            },
        }
        drop(tmp);
        info!("Activity created!");
        JNI_TRUE
    }).unwrap_or(JNI_FALSE);
}

#[no_mangle]
extern "system" fn Java_com_binaryquackers_hbat_MainActivity_bridgeOnDestroy(
  _env: JNIEnv, _activity: JObject) -> jboolean {
    return catch_unwind(|| -> jboolean {
        let mut guard = ACTIVITY_LOCK.write().unwrap();
        *guard = None;
        drop(guard);
        info!("Activity destroyed!");
        JNI_TRUE 
    }).unwrap_or(JNI_FALSE);
}

#[no_mangle]
extern "system" fn Java_com_binaryquackers_hbat_MainActivity_00024MainSurfaceCallback_bridgeSurfaceChanged(
  env: JNIEnv, _callback: JObject, surface: JObject) -> jboolean {
    return catch_unwind(|| -> jboolean {
        let graphics = Graphics::new(env, surface).expect("Failed to initialized graphics!");
        let mut guard = GRAPHICS_MUTEX.lock().unwrap();
        *guard = Some(graphics);
        GRAPHICS_CONDVAR.notify_all();
        drop(guard);
        info!("Surface changed!");
        JNI_TRUE
    }).unwrap_or(JNI_FALSE);
}

#[no_mangle]
extern "system" fn Java_com_binaryquackers_hbat_MainActivity_00024MainSurfaceCallback_bridgeSurfaceDestroyed(
  _env: JNIEnv, _callback: JObject) -> jboolean {
    return catch_unwind(|| -> jboolean {
        let mut guard = GRAPHICS_MUTEX.lock().unwrap();
        *guard = None;
        drop(guard);
        info!("Surface destroyed!");
        JNI_TRUE
    }).unwrap_or(JNI_FALSE);
}

