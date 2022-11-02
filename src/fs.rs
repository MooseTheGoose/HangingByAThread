use std::ops::Drop;
use std::ptr::{null, null_mut, addr_of_mut};
use std::io::{Result, Error, ErrorKind, SeekFrom, Seek, Read, Write};
use std::fs::OpenOptions;
use std::os::unix::io::{FromRawFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use memmap2::*;
use std::convert::AsRef;
use std::path::Path;
use crate::utils::*;
use std::ffi::CString;

// The asset handle and bridge functions
// are an Android-only thing. On other platforms,
// you read assets using filesystems.
#[repr(C)]
struct AssetHandle {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<
        (*mut u8, core::marker::PhantomPinned)>,
}

#[link(name = "hbatandroid")]
extern {
    fn bridge_backendOpenAsset(fname: *const u8) -> *mut AssetHandle;
    fn bridge_backendCloseAsset(asset: *mut AssetHandle);
    fn bridge_backendOpenFdAsset(asset: *const u8, pStart: *mut u64, pLen: *mut u64) -> RawFd;
    fn bridge_backendReadAsset(asset: *mut AssetHandle, buf: *mut u8, len: usize, pReadLen: *mut usize) -> bool;
    fn bridge_backendSeekAsset(asset: *mut AssetHandle, from: i32, to: i64, pSeekPos: *mut u64) -> bool;
}

// Asset that reperesents read-only content that
// can be accessed via a separate filesystem.
struct Asset {
    asset_handle: *mut AssetHandle,
}

impl Asset {
    pub fn open<P: AsRef<Path>>(fname: P) -> Result<Asset> {
        return unsafe {
            let cstr = result_to_io(CString::new(fname.as_ref().as_os_str().as_bytes()), ErrorKind::InvalidData)?;
            let asset_handle = bridge_backendOpenAsset(cstr.as_ptr() as *const u8);
            if asset_handle != null_mut() {
                Ok(Asset{asset_handle: asset_handle})
            } else {
                Err(Error::last_os_error())
            }
        };
    }
}

impl Drop for Asset {
    fn drop(&mut self) {
        if self.asset_handle != null_mut() {
            unsafe { bridge_backendCloseAsset(self.asset_handle); }
            self.asset_handle = null_mut();
        }
    }    
}

impl Read for Asset {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        return unsafe {
            let mut sz = 0 as usize;
            if bridge_backendReadAsset(self.asset_handle, buf.as_mut_ptr(), buf.len(), addr_of_mut!(sz)) {
                Ok(sz)
            } else {
                Err(Error::last_os_error())
            }
        }
    }
}

impl Seek for Asset {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let mut startpos = 0 as u64;
        let to: i64;
        let from = match pos {
            SeekFrom::Start(tmp) => { to = tmp as i64; 0 },
            SeekFrom::Current(tmp) => { to = tmp; 1 },
            SeekFrom::End(tmp) => { to = tmp; 2 },
        };
        return unsafe {
            if bridge_backendSeekAsset(self.asset_handle, from, to, addr_of_mut!(startpos)) {
                Ok(startpos)
            } else {
                Err(Error::last_os_error())
            }
        }
    }
}

// Choose between external and asset
// filesystem when opening file.
pub enum FSType {
    Assets,
    External, 
}

enum FSHandleType {
    HAsset(Asset),
    HExternal(std::fs::File),
}

pub struct File {
    contents: FSHandleType,
}

impl File {
    pub fn open<P: AsRef<Path>>(fname: P, fstype: FSType) -> Result<File> {
        return match fstype {
            FSType::Assets => Ok(File {contents: FSHandleType::HAsset(Asset::open(fname)?)}),
            FSType::External => Ok(File {contents: FSHandleType::HExternal(std::fs::File::open(fname)?)}),
        }; 
    }

    // Only external files may be opened like this.
    pub fn open_options<P: AsRef<Path>>(fname: P, options: &OpenOptions) -> Result<File> {
        return Ok(File {contents: FSHandleType::HExternal(options.open(fname)?)});
    }

    pub fn map<P: AsRef<Path>>(fname: P, fstype: FSType) -> Result<Mmap> {
        match fstype {
            FSType::External => {
                let tmp = std::fs::File::open(fname)?;
                unsafe { Mmap::map(&tmp) }
            },
            FSType::Assets => {
                let mut start = 0 as u64;
                let mut len = 0 as u64;
                unsafe {
                    let cstr = result_to_io(CString::new(fname.as_ref().as_os_str().as_bytes()), ErrorKind::InvalidData)?;
                    let fd = bridge_backendOpenFdAsset(cstr.as_ptr() as *const u8, addr_of_mut!(start), addr_of_mut!(len));
                    if fd < 0 { return Err(Error::last_os_error()); }
                    if len > std::usize::MAX as u64 {
                        return Err(Error::new(ErrorKind::OutOfMemory, "Can't map size > usize::MAX")); 
                    }
                    let sz = len as usize;
                    let fp = std::fs::File::from_raw_fd(fd);
                    MmapOptions::new().len(sz).offset(start).map(&fp)
                }
            },
        }
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        return match self.contents {
            FSHandleType::HAsset(ref mut a) => a.read(buf),
            FSHandleType::HExternal(ref mut f) => f.read(buf),
        };
    }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        return match self.contents {
            FSHandleType::HAsset(ref mut a) => a.seek(pos),
            FSHandleType::HExternal(ref mut f) => f.seek(pos),
        };
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        return match self.contents {
            FSHandleType::HAsset(_) => Err(Error::new(ErrorKind::Unsupported, "HAsset cannot be written to.")),
            FSHandleType::HExternal(ref mut f) => f.write(buf),
        };
    }
    fn flush(&mut self) -> Result<()> {
        return match self.contents {
            FSHandleType::HAsset(_) => Ok(()),
            FSHandleType::HExternal(ref mut f) => f.flush(),
        };
    }
}

