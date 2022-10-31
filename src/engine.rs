#![allow(dead_code)]
use crate::*;
use crate::log::*;
use crate::fs::*;
use std::ptr::addr_of_mut;

#[link(name = "hbatandroid")]
extern {
    fn bridge_backendClearScreen(r: f32, g: f32, b: f32, a: f32);
    fn bridge_backendSwapBuffers();
    fn bridge_backendWindowDimensions(width: *mut i32, height: *mut i32);
}


pub struct Engine {
    width: i32,
    height: i32,
}

impl Engine {
    pub fn refresh_dimensions(&mut self) {
        unsafe {
            bridge_backendWindowDimensions(addr_of_mut!(self.width), addr_of_mut!(self.height));
        }
    }
    pub fn clear_screen(&mut self, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            bridge_backendClearScreen(0.57, 0.70, 0.86, 1.0);
        }
    }
    pub fn swap_buffers(&mut self) {
        unsafe {
            bridge_backendSwapBuffers();
        }
    }
    pub fn update(&mut self) {
        self.refresh_dimensions();
        self.clear_screen(0.57, 0.70, 0.86, 1.0);
        self.swap_buffers();
        log("Hello, World!", LogType::Debug);
        let asset_res = File::map("models-le/SimpleBox.model", FSType::Assets);
        if !asset_res.is_ok() {
            log("Was unable to open asset!", LogType::Warn);
            return;
        }
    }
}

pub static mut G_ENGINE: Engine = Engine {
  width: -1,
  height: -1,
};
