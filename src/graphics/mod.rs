pub mod gl;

// Graphics context for various frameworks
// like OpenGL, Vulkan, etc. These are
// non-exhaustive, so you shouldn't instantiate
// these directly. Instead, use Graphics.get_context()
// to get one for you automatically.
pub enum Context {
    GL(gl::Context),

    // Other Backends I can do if I have the time
    // and energy (probably won't, tbh...)

    // Vulkan,
    // Metal,
    // DX11,
}
