#![allow(dead_code)]
#![allow(unused_imports)]
mod math;

#[cfg_attr(target_os = "android", path="bridge/android/mod.rs")]
mod bridge;

