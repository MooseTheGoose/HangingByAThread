use memmap2::*;
use std::io::{Result, Error, ErrorKind};
use crate::math::*;
use std::convert::AsRef;
use std::path::Path;
use log::*;

const MODEL_MAGIC: u32 = 0x31544248;

// [repr(C)] stuff should be associated with each
// individual shader rather than a model,
// but this works for now...
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

pub struct Model {
    data: Mmap,
    pub transform: Matrix4,
}

impl Model {
    pub fn new(data: Mmap) -> Model {
        let bytes: &[u8] = &data;
        let len = bytes.len();
        let magic = u32::from_ne_bytes(bytes[len-4..len].try_into().unwrap());
        if magic != MODEL_MAGIC { warn!("Invalid magic number in model data"); } 
        return Model {data: data, transform: M4_IDENTITY};
    }
    pub fn data<'a>(&'a self) -> &'a [u8] {
        return &self.data;
    }
}

