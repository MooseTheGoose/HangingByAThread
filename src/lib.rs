mod engine;
mod fs;
mod log;
use std::panic;


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
