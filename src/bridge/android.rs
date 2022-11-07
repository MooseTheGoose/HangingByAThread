use std::os::raw::c_int;

use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixListener;
use libc::*;
use std::fs::*;
use std::ffi::{CString, CStr};
use std::ptr::{null};
use std::sync::Mutex;
use std::path::{PathBuf, Path};
use std::mem::MaybeUninit;
use crate::devcon::conmain;
use std::thread::*;
use std::panic::catch_unwind;
use std::process::abort;
use std::cell::*;
use log::Level;
use log::*;
use android_logger::Config;
use gl;
use memmap2::*;
use std::io::{Read};
use once_cell::sync::{OnceCell, Lazy};
use std::ops::Deref;

use jni::{JNIEnv, JavaVM};
use jni::objects::*;
use jni::sys::*;
use gl::types::*;
use std::mem::size_of;

use crate::math::*;

type LazyCell<T, F = fn() -> T> = Lazy<T, F>;

#[link(name = "EGL")]
extern "C" {
    pub fn eglGetProcAddress(fname: *const c_char) -> *const c_void;
}
fn load_gl() {
    gl::load_with(|s| -> *const c_void {
        let cs = match CString::new(s) {
            Ok(tmp) => tmp,
            Err(_) => return null(),
        };
        let raw = cs.into_raw();
        let addr = unsafe { eglGetProcAddress(raw) };
        let _cs = unsafe { CString::from_raw(raw) };
        return addr;
    });
}
extern "system" fn debug_callback(_source: GLenum, _gltype: GLenum, id: GLuint, _severity: GLenum, _length: GLsizei, message: *const GLchar, _user_param: *mut c_void) {
    let cs = unsafe { CStr::from_ptr(message) };
    match cs.to_str() {
        Ok(s) => error!("GL error: {}, message: {}", id, s),
        Err(_) => error!("In Debug Callback: Got message not UTF-8"),
    };
}
fn gl_err_loop() {
    unsafe {
        let mut err = gl::GetError();
        while err != gl::NO_ERROR {
            error!("GL Error: {}", err);
            err = gl::GetError();
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    JNIError(jni::errors::Error),
    IOError(std::io::Error),
    NumericConversionError,
    JNINotInitialized,
    UTF8DecodeError,
}

impl From<jni::errors::Error> for Error {
    fn from(e: jni::errors::Error) -> Self {
        return Error::JNIError(e);
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(_e: std::str::Utf8Error) -> Self {
        return Error::UTF8DecodeError;
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        return Error::IOError(e);
    }
}

// Cache the Java VM so all threads can access it.
static JVM: OnceCell<JavaVM> = OnceCell::new();

// Make JNI a thread-local variable, since the
// cost of TLS is very small compared to the cost
// of JNI and makes this global thread-safe.
thread_local! {
  static JNI: LazyCell<JNIEnv<'static>> = LazyCell::new(|| {
      let jvm = JVM.get().unwrap();
      return jvm.attach_current_thread_as_daemon()
                .expect("Failed to read JNI from JVM");
  });
}

fn describe_and_clear_jni_exception_helper() -> Result<()> {
    return JNI.with(|jnicell| -> Result<()> {
        let jniref = jnicell.deref();
        if jniref.exception_check()? {
            jniref.exception_describe()?;
            jniref.exception_clear()?;
        }
        Ok(())
    });
}

fn describe_and_clear_jni_exception() {
    let res = describe_and_clear_jni_exception_helper();
    if res.is_err() {
        warn!("Could not describe and clear JNI exception for some reason. Strange...");
    }
}

// We need a global AssetManager to get Android assets.
// We can get a global reference and forget about it.
static ASSET_MANAGER: OnceCell<GlobalRef> = OnceCell::new();

// All the path roots we care about in our game.
// These are initialized if bridgeOnCreate finishes.
static INTERNAL_PATH: OnceCell<PathBuf> = OnceCell::new();

pub fn internal_path() -> &'static Path {
    return INTERNAL_PATH.get().unwrap().as_ref();
}

struct Asset {
    istream: GlobalRef,
}

impl Asset {
    fn read_helper(&mut self, buf: &mut [u8]) -> Result<usize> {
        return JNI.with(|jnicell| -> Result<usize> {
            let jniref = jnicell.deref();
            let buflen: jsize = match buf.len().try_into().ok() {
                Some(l) => l,
                None => { return Err(Error::NumericConversionError); }
            };
            let bytearray = jniref.new_byte_array(buflen)?;
            let arrobj: JObject = unsafe { std::mem::transmute(bytearray) };
            let bytearray_val = JValue::Object(arrobj);
            let istream_obj = self.istream.as_obj();
            let nbytes_i32 = std::cmp::max(jniref.call_method(istream_obj, "read", "([B)I", &[bytearray_val])?.i()?, 0);
            Ok({
                let jnislice = unsafe { std::slice::from_raw_parts_mut(buf.as_ptr() as *mut jbyte, buf.len()) };
                let nbytes = nbytes_i32 as usize;
                jniref.get_byte_array_region(bytearray, 0, &mut jnislice[..nbytes])?;
                nbytes
            })
        });
    }
}

impl std::io::Read for Asset {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let res = self.read_helper(buf);
        describe_and_clear_jni_exception();
        return match res {
            Ok(r) => Ok(r),
            Err(e) => {
                match e {
                    Error::JNIError(_) => Err(std::io::Error::new(std::io::ErrorKind::Other, "JNI error occurred")),
                    Error::NumericConversionError => Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Read too big for JNI")),
                    _ => Err(std::io::Error::last_os_error()),
                }
            }
        };
    }
}

impl Asset {
    fn open_helper<P: AsRef<Path>>(path: P) -> Result<Asset> {
        return JNI.with(|jnicell| -> Result<Asset> {
            let jniref = jnicell.deref();
            let res = {
                let mgrglobal = ASSET_MANAGER.get().unwrap();
                let mgr = mgrglobal.as_obj();
                let fname_str = match path.as_ref().to_str() {
                    Some(s) => Ok(s),
                    None => Err(Error::UTF8DecodeError),
                }?;
                let jstr = jniref.new_string(fname_str)?;
                let fname_obj = JValue::Object(JObject::from(jstr));
                jniref.new_global_ref(jniref.call_method(mgr, "open", "(Ljava/lang/String;)Ljava/io/InputStream;", &[fname_obj])?.l()?)
            };
            Ok(Asset { istream: res? })
        });
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Asset> {
        let res = Self::open_helper(path);
        describe_and_clear_jni_exception();
        return res;
    } 

    fn map_helper<P: AsRef<Path>>(path: P) -> Result<Mmap> {
        return JNI.with(|jnicell| -> Result<Mmap> {
            let jniref = jnicell.deref();
            let mgrglobal = ASSET_MANAGER.get().unwrap();
            let mgr = mgrglobal.as_obj();
            let fname_str = match path.as_ref().to_str() {
                Some(s) => Ok(s),
                None => Err(Error::UTF8DecodeError),
            }?;
            let jstr = jniref.new_string(fname_str)?;
            let fname_obj = JValue::Object(JObject::from(jstr));
            let afd_opt = jniref.call_method(mgr, "openFd", "(Ljava/lang/String;)Landroid/content/res/AssetFileDescriptor;", &[fname_obj]).ok();
            if afd_opt.is_none() {
                // Try again via creating an Asset instead of using openFd.
                // The exception must also be cleared, but don't report it.
                match jniref.exception_check() {
                    Ok(excp) => if excp { let _ = jniref.exception_clear(); },
                    _ => {},
                };
                let mut asset = Self::open(path)?;
                let mut data = vec![];
                asset.read_to_end(&mut data)?;
                return {
                    let mut mmap_mut = MmapOptions::new().len(data.len()).map_anon()?;
                    let mmap_slice: &mut [u8] = &mut mmap_mut;
                    mmap_slice.clone_from_slice(&data[..]);
                    Ok(mmap_mut.make_read_only()?)
                };
            }
            let afd = afd_opt.unwrap().l()?;
            let start = jniref.call_method(afd, "getStartOffset", "()J", &[])?.j()?;
            let len: usize = match jniref.call_method(afd, "getLength", "()J", &[])?.j()?.try_into().ok() {
                Some(u) => Ok(u),
                None => Err(Error::NumericConversionError),
            }?;
            let pfd = jniref.call_method(afd, "getParcelFileDescriptor", "()Landroid/os/ParcelFileDescriptor;", &[])?.l()?;
            let fd = jniref.call_method(pfd, "detachFd", "()I", &[])?.i()?;
            return unsafe { 
                let res = MmapOptions::new().offset(start as u64).len(len).map(fd);
                close(fd);
                Ok(res?)
            };
        });
    }

    pub fn map<P: AsRef<Path>>(path: P) -> Result<Mmap> {
        let res = Self::map_helper(path);
        describe_and_clear_jni_exception();
        return res;
    }
}

struct Shader {
    vs_source: Mmap,
    fs_source: Mmap,
    prog: GLuint,
}

#[repr(C)]
pub struct Mesh {
    pub num_vertices: u32,
    pub num_faces: u32,
}

#[repr(C)]
pub struct Node {
    pub num_meshes: u32,
    pub num_children: u32,
    pub transform3: Matrix3,
    pub translate3: Vector3,
}

#[repr(C)]
pub struct ModelTable {
    magic: u32,
    shader_id: u32,
    mesh_data_start: u32,
    mesh_indices_start: u32,
    mesh_array_start: u32,
}

#[repr(C)]
pub struct MeshArray {
    pub voffset: u32,
    pub foffset: u32,
    pub nverts: u32,
    pub nfaces: u32,
}

pub struct Model {
    model_data: Mmap,
    vertex_data: GLuint,
    index_data: GLuint,
    vertex_data_size: usize,
    index_data_size: usize, 
    pub transform: Matrix4,
}

// Yes... I know the sins to alignment I'm committing for.
// both Model and Shader. I'll panic on this later but I'm
// too lazy right now.
impl Model {
    pub unsafe fn reload(&mut self) {
        let mapslice = &self.model_data;
        let table = (mapslice[..size_of::<ModelTable>()].as_ptr() as *const ModelTable).as_ref().unwrap();
        let data = &mapslice[size_of::<ModelTable>()..];
        let v_begin = (table.mesh_data_start * 4) as usize;
        let v_end = (table.mesh_indices_start * 4) as usize;
        let vertex_slice = &data[v_begin..v_end];
        let _debug_vertex: &[f32] = std::slice::from_raw_parts(vertex_slice.as_ptr() as *const f32, vertex_slice.len() / 4);
        let indices_slice = &data[v_end..];
        let _debug_index: &[u32] = std::slice::from_raw_parts(indices_slice.as_ptr() as *const u32, indices_slice.len() / 4);
        self.vertex_data_size = vertex_slice.len();
        self.index_data_size = indices_slice.len();
        let mut tmp: [GLuint; 2] = [0; 2];
        gl::GenBuffers(2, &mut tmp as *mut GLuint);
        self.vertex_data = tmp[0];
        self.index_data = tmp[1];
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_data);
        gl::BufferData(gl::ARRAY_BUFFER, vertex_slice.len() as GLsizeiptr, vertex_slice.as_ptr() as *const c_void, gl::STATIC_DRAW);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_data);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, indices_slice.len() as GLsizeiptr, indices_slice.as_ptr() as *const c_void, gl::STATIC_DRAW);
        // We do technically still need nodes, but this should
        // just be a couple painful but quick page faults.
        // If it's bad enough, I can make a custom Mmap struct.
        let _ = Mmap::advise(&self.model_data, Advice::DontNeed);
    }
    pub unsafe fn new(name: &str) -> Result<Model> {
        let root = "shared/models";
        let loc = format!("{}/{}.model", root, name);
        let data = Asset::map(loc)?;
        let mut model = Model {model_data: data, vertex_data: 0, vertex_data_size: 0, index_data: 0, index_data_size: 0, transform: M4_IDENTITY};
        model.reload();
        return Ok(model);
    }
    pub fn vdata(&self) -> GLuint {
        return self.vertex_data;
    }
    pub fn fdata(&self) -> GLuint {
        return self.index_data;
    }
    pub fn mdata<'a>(&'a self) -> &'a [u8] {
        return &self.model_data;
    }
    pub fn table<'a>(&'a self) -> &'a ModelTable {
        return unsafe { (self.mdata().as_ptr() as *const ModelTable).as_ref().unwrap() };
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        unsafe {
            let mut tmp: [GLuint; 2] = [0; 2];
            tmp[0] = self.vertex_data;
            tmp[1] = self.index_data;
            gl::DeleteBuffers(2, &tmp as *const GLuint);
        }
    }
}

impl Shader {
    // At the moment, this will just panic
    // if the shader can't do what it needs to.
    unsafe fn print_program_log(program: GLuint) {
        let mut tmp: [u8; 1024] = [0; 1024];
        let mut l: GLsizei = 0;
        gl::GetProgramInfoLog(program, tmp.len() as GLsizei, &mut l as *mut GLsizei, tmp.as_mut_ptr() as *mut GLchar);
        match std::str::from_utf8(&tmp[0..l as usize]) {
            Ok(s) => error!("{}", s),
            Err(_) => error!("Unable to get Shader Log!"),
        }
    }
    unsafe fn print_shader_log(shader: GLuint) {
        let mut tmp: [u8; 1024] = [0; 1024];
        let mut l: GLsizei = 0;
        gl::GetShaderInfoLog(shader, tmp.len() as GLsizei, &mut l as *mut GLsizei, tmp.as_mut_ptr() as *mut GLchar);
        match std::str::from_utf8(&tmp[0..l as usize]) {
            Ok(s) => error!("{}", s),
            Err(_) => error!("Unable to get Shader Log!"),
        }
    }
    unsafe fn load_subshader(typ: GLuint, src: &[u8]) -> GLuint {
        let s = gl::CreateShader(typ);
        gl::ShaderSource(s, 1, [src.as_ptr() as *const c_char].as_ptr(), [src.len() as GLint].as_ptr());
        gl::CompileShader(s);
        return s;
    }
    pub unsafe fn reload(&mut self) {
        let v_shader = Self::load_subshader(gl::VERTEX_SHADER, &self.vs_source);
        Self::print_shader_log(v_shader);
        let f_shader = Self::load_subshader(gl::FRAGMENT_SHADER, &self.fs_source);
        Self::print_shader_log(f_shader);
        self.prog = gl::CreateProgram();
        gl::AttachShader(self.prog, v_shader);
        gl::AttachShader(self.prog, f_shader);
        gl::LinkProgram(self.prog);
        Self::print_program_log(self.prog);
        gl::DeleteShader(v_shader);
        gl::DeleteShader(f_shader);
        let _ = Mmap::advise(&self.vs_source, Advice::DontNeed);
        let _ = Mmap::advise(&self.fs_source, Advice::DontNeed);
    }
    pub unsafe fn new(name: &str) -> Result<Shader> {
        let root = format!("android/shaders/{}", name);
        let vsloc = format!("{}/{}.vert", root, name);
        let fsloc = format!("{}/{}.frag", root, name);
        let vsmap = Asset::map(vsloc)?;
        let fsmap = Asset::map(fsloc)?;
        let mut shdr = Shader { vs_source: vsmap, fs_source: fsmap, prog: 0 };
        shdr.reload();
        return Ok(shdr);
    }
    pub unsafe fn draw(&self, model: &Model) {
        let data = model.mdata();
        let vbuffer = model.vdata();
        let ibuffer = model.fdata();
        let table = model.table();
        let other = &data[size_of::<ModelTable>()..];
        let mesh_ofs = (table.mesh_data_start as usize) * 4;
        let table_ofs = (table.mesh_array_start as usize) * 4;
        let table_len = (mesh_ofs - table_ofs) / size_of::<MeshArray>();
        let table_bytes = &other[table_ofs..mesh_ofs];
        let mesh_arrays: &[MeshArray] =
            std::slice::from_raw_parts(table_bytes.as_ptr() as *const MeshArray, table_len as usize);
        gl::UseProgram(self.prog);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibuffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbuffer);
        let a = &mesh_arrays[0];
        info!("{}, {}, {}, {}", a.voffset, a.foffset, a.nverts, a.nfaces);
        let attrib_loc = gl::GetAttribLocation(self.prog, "position\x00".as_ptr() as *const GLchar) as u32;
        gl::EnableVertexAttribArray(attrib_loc);
        gl::VertexAttribPointer(
            attrib_loc,
            3,
            gl::FLOAT,
            gl::FALSE,
            0,
            std::mem::transmute((a.voffset * 4) as usize),
        );
        // gl::DrawElements(gl::TRIANGLES, (a.nfaces * 3) as i32, gl::UNSIGNED_INT, ((a.foffset * 4) as usize) as *const c_void);
        gl::DrawArrays(gl::TRIANGLES, 0, a.nverts as i32);
        gl::ValidateProgram(self.prog);
        Shader::print_program_log(self.prog);
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.prog);
        }
    }
}

// I must use some function in this bridge
// in the C++ file for Android Studio. Otherwise,
// this static library gets optimized out.
#[no_mangle]
pub extern "C" fn hbat_bridge_stub() {
}

fn init_paths(ctx: JObject) {
    JNI.with(|jnicell| -> Result<()> {
        let jniref = jnicell.deref();
        let files_dir = jniref.call_method(ctx, "getFilesDir", "()Ljava/io/File;", &[])?.l()?;
        let abs_path = JString::from(jniref.call_method(files_dir, "getAbsolutePath", "()Ljava/lang/String;", &[])?.l()?);
        let buf = PathBuf::from(
            jniref.get_string(abs_path)?.to_str()?
        );
        INTERNAL_PATH.set(buf).unwrap();
        Ok(())
    }).expect("Failed to get file roots from JNI");
}

fn init_asset_manager(ctx: JObject) {
    JNI.with(|jnicell| -> Result<()> {
        let jniref = jnicell.deref();
        let asset_manager = jniref.call_method(ctx, "getAssets", "()Landroid/content/res/AssetManager;", &[])?.l()?;
        let global_manager = jniref.new_global_ref(asset_manager)?;
        ASSET_MANAGER.set(global_manager).unwrap();
        Ok(())
    }).expect("Failed to get AssetManager from JNI");
}

// Initialize the runtime here with this JNI method.
// Return if this initialization was successful or not.
#[no_mangle]
pub extern "system" fn Java_com_binaryquackers_hbat_MainActivity_bridgeOnCreate(
    env: JNIEnv, ctx: JObject) -> jboolean {
    // Set up first-time initialization here!
    return catch_unwind(|| -> jboolean{
        android_logger::init_once(
            Config::default().with_min_level(Level::Trace),
        );
        JVM.set(env.get_java_vm().expect("Unable to get JVM from JNI!")).unwrap();
        init_paths(ctx);
        init_asset_manager(ctx);
        load_gl();
        // spawn(|| { devcon_loop(); });
        JNI_TRUE
    }).unwrap_or(JNI_FALSE);
}

// Called when EGL context is lost.
// Recreate resources associated with last CTX
#[no_mangle]
pub extern "system" fn Java_com_binaryquackers_hbat_MainGLSurfaceView_00024MainGLRenderer_bridgeOnSurfaceCreated(
    _env: JNIEnv, _renderer: JObject) -> jboolean {
    return catch_unwind(|| -> jboolean {
        unsafe {
  	    gl::Enable(gl::BLEND);
	    gl::Enable(gl::DEPTH_TEST);
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
        }
        info!("onSurfaceCreated");
        JNI_TRUE
    }).unwrap_or(JNI_FALSE);
}

#[no_mangle]
pub extern "system" fn Java_com_binaryquackers_hbat_MainGLSurfaceView_00024MainGLRenderer_bridgeOnDrawFrame(
    _env: JNIEnv, _renderer: JObject) -> jboolean {
    return catch_unwind(|| -> jboolean { 
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            let mut model = Model::new("SimpleBox").expect("Can't open simplebox?");
            let shader = Shader::new("basic").expect("Can't open basic?");
            model.transform = model.transform * 0.25;
            model.transform.v4.w = 1.0;
            shader.draw(&model);
            gl_err_loop();
            info!("onDrawFrame");
        }
        JNI_TRUE
    }).unwrap_or(JNI_FALSE);
}

#[no_mangle]
pub extern "system" fn Java_com_binaryquackers_hbat_MainGLSurfaceView_00024MainGLRenderer_bridgeOnSurfaceChanged(
  _env: JNIEnv, _renderer: JObject, width: jint, height: jint) -> jboolean {
    return catch_unwind(|| -> jboolean {
        unsafe {
            gl::Viewport(0, 0, width, height);
            // Also reinstantiate everything at this point.
            info!("onSurfaceChanged");
        }
        JNI_TRUE
    }).unwrap_or(JNI_FALSE);
}

fn devcon_loop() {
    let listener_result = {
        let unixpath = internal_path().join("devcon");
        let _ = remove_file(unixpath.as_path()); 
        UnixListener::bind(unixpath.as_path())
    };
    match listener_result {
        Ok(listener) => {
            info!("Waiting for devcon clients...");
            for stream_result in listener.incoming() {
               match stream_result {
                   Ok(stream) => {
                       let fd = stream.as_raw_fd();
                       unsafe {
                           dup2(fd, 2);
                           dup2(fd, 1);
                           dup2(fd, 0);
                       }
                       conmain();
                   },
                   Err(_) => { 
                       warn!("Connection to devcon didn't succeed...");
                   }
               } 
            }
        },
        Err(e) => {
            error!("Unable to create UNIX socket for devcon! {:?}", e);
        }
    };
}


