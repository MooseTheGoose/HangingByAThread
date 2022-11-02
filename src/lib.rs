#![allow(dead_code)]
#![allow(unused_imports)]
#[macro_use] extern crate log;
extern crate android_logger;
mod engine;
mod fs;
mod utils;
mod graphics;
mod math;
mod devcon;
use std::panic;

use log::Level;
use android_logger::{Config,FilterBuilder};

#[no_mangle]
pub extern "C" fn bridge_frontendDevconEntry() {
    let _ = panic::catch_unwind(|| {
        crate::devcon::conmain();
    });
}

#[no_mangle]
pub extern "C" fn bridge_frontendInit() -> bool {
    return panic::catch_unwind(|| {
        android_logger::init_once(
            Config::default()
                .with_min_level(Level::Trace)
                .with_tag("HangingByAThread"));
    }).is_ok();
}

#[no_mangle]
pub extern "C" fn bridge_frontendUpdate() -> bool {
    // Rust stack unwinding undefined across
    // language boundaries, so we must catch
    // the unwind. Thankfully, this is the only
    // function the backend calls.
    return panic::catch_unwind(|| {
        unsafe { engine::G_ENGINE.update(); }
    }).is_ok();
}
