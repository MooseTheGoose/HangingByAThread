
//pub mod android;
//pub mod asset;
//pub mod devcon;
pub mod activity;
pub mod globals;
pub mod bindings;

#[path="graphics/mod.rs"]
pub mod graphics;

use once_cell::sync::{OnceCell, Lazy};
use std::fmt::format;

type LazyCell<T, F = fn() -> T> = Lazy<T, F>;
type SyncOnceCell<T> = OnceCell<T>;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    JNIError(jni::errors::Error),
    IOError(std::io::Error),
    EGLError(khronos_egl::Error),
    EGLNoDisplay,
    EGLInvalidLibrary,
    NoEGLConfigs,
    NumericConversionError,
    JNINotInitialized,
    UTF8DecodeError,
    WrongThread,
    NoWindow,
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

impl From<khronos_egl::Error> for Error {
    fn from(e: khronos_egl::Error) -> Self {
        return Error::EGLError(e);
    }
}

impl From<Error> for std::io::Error {
    fn from(e: Error) -> Self {
        return match e {
           Error::IOError(ioe) => ioe,
           _ => std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}",e)),
        };
    }
}
