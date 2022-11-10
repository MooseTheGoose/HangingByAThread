use crate::bridge::graphics::*;
use std::sync::mpsc::{Sender,Receiver};
use std::ffi::c_void;
use log::*;
use gl::*;

pub fn render(graphics: &mut Graphics) {
    match unsafe {graphics.get_context()} {
        Ok(_) => {
            info!("Got the context!");
            unsafe {
                gl::Viewport(0, 0, graphics.width, graphics.height);
                gl::ClearColor(1.0, 1.0, 1.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }
        }
        Err(e) => {
            error!("Did not get the context! {:?}", e);
        },
    };
}
