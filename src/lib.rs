#![allow(dead_code)]
#![allow(unused_imports)]
#[macro_use] extern crate log;
extern crate android_logger;
extern crate gl;
mod devcon;

#[cfg_attr(target_os = "android", path="bridge/android.rs")]
mod bridge;


