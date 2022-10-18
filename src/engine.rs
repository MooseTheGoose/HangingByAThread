#![allow(dead_code)]
use crate::*;
use std::ptr;

pub struct Engine {
    width: i32,
    height: i32,
}

impl Engine {
    pub fn update(&mut self) {
        unsafe {
            bridge_backendWindowDimensions(ptr::addr_of_mut!(self.width), ptr::addr_of_mut!(self.height));
            bridge_backendClearScreen(0.57, 0.70, 0.86, 1.0);
            bridge_backendSwapBuffers();
        };
    }
}

pub static mut G_ENGINE: Engine = Engine {
  width: -1,
  height: -1,
};
