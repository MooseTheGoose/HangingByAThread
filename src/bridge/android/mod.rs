
//pub mod android;
//pub mod asset;
//pub mod devcon;
pub mod activity;
pub mod globals;
pub mod bindings;
pub mod graphics;

use once_cell::sync::{OnceCell, Lazy};

type LazyCell<T, F = fn() -> T> = Lazy<T, F>;
type SyncOnceCell<T> = OnceCell<T>;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    JNIError(jni::errors::Error),
    IOError(std::io::Error),
    NumericConversionError,
    JNINotInitialized,
    UTF8DecodeError,
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

