use memmap2::*;
use std::io::{Result, Error, ErrorKind};
use crate::fs::*;
use crate::utils::*;
use std::convert::AsRef;
use std::path::Path;
use log::*;

const MODEL_MAGIC: u32 = 0x31544248;

pub struct Model {
    data: Mmap,
    node_start: u32,
}

impl Model {
    pub fn new(data: Mmap) -> Model {
        let bytes: &[u8] = &data;
        let len = bytes.len();
        if len < 8 {
            warn!("Model data to small to parse. Assuming default values.");
            return Model {data: data, node_start: 0};
        }
        let magic = u32::from_ne_bytes(bytes[len-4..len].try_into().unwrap());
        if magic != MODEL_MAGIC { warn!("Invalid magic number in model data"); } 
        let node_start = u32::from_ne_bytes(bytes[len-8..len-4].try_into().unwrap());
        return Model {data: data, node_start: node_start};
    }
    pub fn open<P: AsRef<Path>>(path: P, fstype: FSType) -> Result<Model> {
        let map = File::map(path, fstype)?;
        Ok(Model::new(map))
    }
}

