use core::marker::{PhantomData, PhantomPinned};
use jni::JNIEnv;
use jni::objects::{JObject,JString};
use std::ptr::{null_mut,null,NonNull};
use core::ffi::c_void;
use crate::bridge::{Result,Error};
use crate::graphics::{self, Context};
use std::rc::Rc;
use std::ops::Deref;
use raw_window_handle::{
    HasRawWindowHandle,RawWindowHandle,RawDisplayHandle,
    AndroidNdkWindowHandle,AndroidDisplayHandle
};
use libloading::Library;
use khronos_egl as egl;
use egl::*;
use gl;
use std::ffi::CString;
use crate::bridge::globals::*;
use std::boxed::Box;
use std::ffi::CStr;
use std::ffi::c_char;

#[repr(C)]
pub struct ANativeWindow {
    _data: [u8; 0],
    _marker:
        PhantomData<(*mut u8, PhantomPinned)>,
}

#[link(name = "android")]
extern {
    fn ANativeWindow_fromSurface(env: JNIEnv, surface: JObject) -> *mut ANativeWindow;
    fn ANativeWindow_acquire(handle: *mut ANativeWindow);
    fn ANativeWindow_release(handle: *mut ANativeWindow);
    fn ANativeWindow_getWidth(handle: *mut ANativeWindow) -> i32;
    fn ANativeWindow_getHeight(handle: *mut ANativeWindow) -> i32;
}

// Wrapper for Window object to
// impl Drop and HasRawWindowHandle.
// Also implement Send, even though it's
// not actually send, since we need to send
// it to the main loop and back.
pub struct WWindow(*mut ANativeWindow);

unsafe impl Send for WWindow {}

impl WWindow {
    pub unsafe fn get_raw(&self) -> *mut c_void {
        return self.0 as *mut c_void;
    }
}

impl Drop for WWindow {
    fn drop(&mut self) {
        unsafe {
            if self.0 != null_mut() {
                ANativeWindow_release(self.0);
            }
        }
    }
}

unsafe impl HasRawWindowHandle for WWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut raw = AndroidNdkWindowHandle::empty();
        raw.a_native_window = self.0 as *mut c_void;
        return RawWindowHandle::AndroidNdk(raw);
    }
}

// Wrapper around EGL surface for impl Drop
pub struct WEGLSurface(pub egl::Surface, pub egl::Display,
  pub Rc<egl::Instance<Dynamic<libloading::Library,egl::EGL1_0>>>);

impl Drop for WEGLSurface {
    fn drop(&mut self) {
        match self.2.destroy_surface(self.1,self.0) {
            Err(e) => log::error!("Failed to destroy EGL surface: {:?}", e), 
            Ok(_) => {},
        };
    }
}

// Wrapper around context 
pub struct WEGLContext(pub egl::Context, pub egl::Display,
    pub Rc<egl::Instance<Dynamic<libloading::Library,egl::EGL1_0>>>);

impl Drop for WEGLContext {
    fn drop(&mut self) {
        match self.2.destroy_context(self.1,self.0) {
            Err(e) => log::error!("Failed to destroy EGL context: {:?}", e),
            Ok(_) => {},
        };
    }
}

pub struct PlatformGLContext {
    api: Rc<egl::Instance<Dynamic<libloading::Library,egl::EGL1_0>>>,
    display: egl::Display,
    surface: WEGLSurface,
    egl_ctx: WEGLContext,
    pub context: Context,
}

impl PlatformGLContext {
    pub unsafe fn swap_buffers(&mut self) {
        match self.api.swap_buffers(self.display, self.surface.0) {
            Err(e) => { log::warn!("Failed to swap buffers: {:?}", e); }
            _ => {}
        };
    }
}

unsafe impl Send for PlatformGLContext {}

pub struct Graphics {
    window: WWindow,
    context: Option<PlatformGLContext>,
    pub width: i32,
    pub height: i32,
}

impl Graphics {
    pub fn new(env: JNIEnv, surface: JObject) -> Result<Graphics> {
        let w = unsafe { ANativeWindow_fromSurface(env, surface) };
        return match NonNull::new(w) {
            Some(window) => unsafe { Self::from_window(window) },
            None => Err(Error::NoWindow),
        };
    }
    pub unsafe fn from_window(handle: NonNull<ANativeWindow>) -> Result<Graphics> {
        let width = ANativeWindow_getWidth(handle.as_ptr());
        let height = ANativeWindow_getHeight(handle.as_ptr());
        let window = WWindow(handle.as_ptr());
        let ctx = Graphics {
            window: window,
            context: None,
            width: width,
            height: height,
        };
        return Ok(ctx);
    }
    pub unsafe fn get_gl_context<'a>(&'a mut self) -> Result<PlatformGLContext> {
        let egl_api = Rc::new(match egl::DynamicInstance::<egl::EGL1_0>::load_required() {
            Ok(api) => Ok(api),
            Err(_e) => Err(Error::EGLInvalidLibrary),
        }?);
        let display = match egl_api.get_display(egl::DEFAULT_DISPLAY) {
            Some(d) => Ok(d),
            None => Err(Error::EGLNoDisplay),
        }?;
        egl_api.initialize(display)?;
        let attrs = [
            egl::RED_SIZE, 8,
            egl::GREEN_SIZE, 8,
            egl::BLUE_SIZE, 8,
            egl::RENDERABLE_TYPE, egl::OPENGL_ES2_BIT,
            egl::NONE,
        ];
        let cfg = match egl_api.choose_first_config(display, &attrs)? {
            Some(c) => c,
            None => return Err(Error::NoEGLConfigs),
        };
        // To start, only use OpenGL 2 so we maintain compatibility.
        // Optimize later for OpenGL 3 contexts.
        let supported_versions = [
            (4,0),
            (3,0),
            (2,0),
        ];
        let mut ctx_unwrapped: std::result::Result<egl::Context, egl::Error> = Err(egl::Error::BadContext);
        for version in &supported_versions {
            let context_attributes = [
                egl::CONTEXT_MAJOR_VERSION, version.0,
                egl::CONTEXT_MINOR_VERSION, version.1,
                egl::NONE,
            ];
            ctx_unwrapped = egl_api.create_context(display, cfg, None, &context_attributes);
            if ctx_unwrapped.is_ok() { break; }
        }
        let ctx = WEGLContext(ctx_unwrapped?, display, egl_api.clone());
        let surface_attributes = [
            egl::NONE,
        ];
        let surface_unwrapped = egl_api.create_window_surface(
            display,
            cfg,
            self.window.get_raw() as *mut c_void,
            Some(&surface_attributes),
        )?;
        let surface = WEGLSurface(surface_unwrapped, display, egl_api.clone());
        let _ = egl_api.make_current(display, Some(surface.0), Some(surface.0), Some(ctx.0));
        gl::load_with(|s| -> *const _ {
            return match egl_api.get_proc_address(s) {
                Some(p) => p as *const c_void,
                None => null() as *const c_void,
            };
        });
        // If the glGetIntegerv call here fails, we know
        // that this context is version 2.0 or 1.1. We require
        // OpenGL ES 2.0+, so if it fails, we'll assume it's 2.0.
        let mut major: i32 = 2;
        let mut minor: i32 = 0;
        gl::GetIntegerv(gl::MAJOR_VERSION, &mut major as *mut i32);
        let err = gl::GetError();
        if err == gl::NO_ERROR {
            gl::GetIntegerv(gl::MINOR_VERSION, &mut minor as *mut i32); 
        } else {
            major = 2;
        }
        return Ok(PlatformGLContext {
            api: egl_api,
            display: display,
            surface: surface,
            egl_ctx: ctx,
            context: Context::GL(crate::graphics::gl::Context {
                major: major as u8,
                minor: minor as u8,
            }),
        });
    }
    pub unsafe fn get_context<'a>(&'a mut self) -> Result<&'a Context> {
        let valid_id = LOCAL_THREAD_ID.with(|idcell| -> bool {
            let id_opt = idcell.get();
            let renderer_opt = RENDERER_THREAD_ID.get();
            return id_opt.is_none() || renderer_opt.is_none()
                   || (id_opt.unwrap() == renderer_opt.unwrap());
        });
        if !valid_id {
            return Err(Error::WrongThread);
        }
        if self.context.is_none() {
            self.context = Some(self.get_gl_context()?);
        };
        return Ok(&self.context.as_ref().unwrap().context);
    }
    pub unsafe fn swap_buffers(&mut self) {
        if self.context.is_some() {
            self.context.as_mut().unwrap().swap_buffers();
        }
    }
}
