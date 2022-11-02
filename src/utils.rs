use std::ffi::{CString, CStr};
use std::io::{Result, Error, ErrorKind};
use std::marker::{Send, Sync};
use std::ops::Range;

pub fn result_to_io<T,E: std::error::Error+Send+Sync+'static>
  (res: std::result::Result<T,E>, kind: ErrorKind) -> Result<T> {
    return match res {
        Ok(r) => Ok(r),
        Err(e) => Err(Error::new(kind, e)),
    };
}

/*
pub fn slice_clamp<T>(arr: &[T], range: Range) {
}
*/

