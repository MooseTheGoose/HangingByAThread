mod engine;

#[link(name = "hbatandroid")]
extern {
    fn bridge_backendClearScreen(r: f32, g: f32, b: f32, a: f32);
    fn bridge_backendSwapBuffers();
    fn bridge_backendWindowDimensions(width: *mut i32, height: *mut i32);
}

#[no_mangle]
pub extern "C" fn bridge_frontendUpdate() {
    unsafe { engine::G_ENGINE.update(); }
}
