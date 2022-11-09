use core::marker::{PhantomData, PhantomPinned};
use jni::JNIEnv;
use jni::objects::{JObject,JString};
use std::ptr::{null_mut,null,NonNull};
use crate::bridge::{Result,Error};
use std::sync::Arc;
use crate::bridge::globals::VULKAN_LIBRARY;
use vulkano::instance::Instance;
use vulkano::swapchain::{Surface};
use std::ops::Deref;

#[repr(C)]
pub struct ANativeWindow {
    _data: [u8; 0],
    _marker:
        PhantomData<(*mut u8, PhantomPinned)>,
}

pub struct Window(NonNull<ANativeWindow>);

#[link(name = "android")]
extern {
    fn ANativeWindow_fromSurface(env: JNIEnv, surface: JObject) -> *mut ANativeWindow;
    fn ANativeWindow_acquire(handle: *mut ANativeWindow);
    fn ANativeWindow_release(handle: *mut ANativeWindow);
    fn ANativeWindow_getWidth(handle: *mut ANativeWindow) -> i32;
    fn ANativeWindow_getHeight(handle: *mut ANativeWindow) -> i32;
}

impl Clone for Window {
    fn clone(&self) -> Self {
        let w = self.0.as_ptr();
        unsafe { ANativeWindow_acquire(w); };
        return Window(NonNull::new(w).unwrap());
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe { ANativeWindow_release(self.0.as_ptr()); }
    }
}

// We keep the ANativeWindow private and
// use only thread-safe operations like
// releasing and graphics surface creation,
// so this is both Sync and Send.
unsafe impl Send for Window {}
unsafe impl Sync for Window {}

// A general context object for graphics.
// This contains a window handle and graphics
// API stuff...
pub struct Context {
    window: Arc<Window>,
    pub width: i32,
    pub height: i32,
}

impl Context {
    pub fn new(env: JNIEnv, surface: JObject) -> Result<Context> {
        let w = unsafe { ANativeWindow_fromSurface(env, surface) };
        return match NonNull::new(w) {
            Some(window) => Self::from_window(window),
            None => Err(Error::NoWindow),
        };
    }
    pub fn from_window(handle: NonNull<ANativeWindow>) -> Result<Context> {
        let width = unsafe { ANativeWindow_getWidth(handle.as_ptr()) };
        let height = unsafe { ANativeWindow_getHeight(handle.as_ptr()) };
        let window_ref = Arc::new(Window(handle));
        let ctx = Context {
            window: window_ref,
            width: width,
            height: height,
        };
        return Ok(ctx);
    }
}
