//! Provide file operation
//! # Example
//! ```
//! use::rtt_rs::fs;
//! let f = fs::File::with_options().read(true).append(true).open("/usr/xxx").unwrap();
//! f.write(Vec::from("abcdefg".as_bytes())).unwrap();
//! f.sync().unwrap();
//! ```
use crate::alloc::fmt::Formatter;
use crate::base::{CString, RTTError};
use crate::fmt;
use crate::string::String;
use crate::vec::Vec;

pub struct File {
    fd: i32,
    flag: i32,
    path: String,
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

const FILE_ERR: i32 = 0;

mod rttbase {
    extern "C" {
        pub(crate) fn open(file: *const u8, flags: i32) -> i32;
        pub(crate) fn close(fd: i32) -> i32;
        pub(crate) fn read(fd: i32, buf: *mut u8, len: usize) -> i32;
        pub(crate) fn write(fd: i32, buf: *const u8, len: usize) -> i32;
        pub(crate) fn rename(old: *const u8, new: *const u8) -> i32;
        pub(crate) fn unlink(path_name: *const u8) -> i32;
        pub(crate) fn fsync(fd: i32) -> i32;
    }
}

// Open flag
const READ_ONLY: i32 = 0;
const WRITE_ONLY: i32 = 1;
const READ_WRITE: i32 = 2;
const CREATE: i32 = 0x0200;
const APPEND: i32 = 0x0008;
const TRUNCATION: i32 = 0x0400;

impl File {
    pub fn raw_open(name: &str, _flag: i32) -> Result<File, RTTError> {
        unsafe {
            let c: CString = name.into();
            let _fd = rttbase::open(c.into(), _flag);
            if _fd == -1 {
                Err(RTTError::FileOpenFailed)
            } else {
                Ok(File {
                    fd: _fd,
                    flag: _flag,
                    path: String::from(name),
                })
            }
        }
    }

    pub fn open(path: &str) -> Result<File, RTTError> {
        Self::raw_open(path, READ_ONLY)
    }

    pub fn create(path: &str) -> Result<File, RTTError> {
        Self::raw_open(path, READ_ONLY | CREATE)
    }

    pub fn with_options() -> OpenOptions {
        OpenOptions::new()
    }

    pub fn write(&self, buf: Vec<u8>) -> Result<(), RTTError> {
        unsafe {
            let w_len = rttbase::write(self.fd, buf.as_ptr(), buf.len());
            return if w_len == buf.len() as i32 {
                Ok(())
            } else {
                Err(RTTError::FileWriteFailed)
            };
        }
    }

    pub fn read(&self, len: usize) -> Result<Vec<u8>, RTTError> {
        let mut temp = Vec::new();
        temp.resize(len, 0_u8);

        let r_len;
        unsafe {
            r_len = rttbase::read(self.fd, temp.as_mut_ptr(), len);
        }
        return if r_len <= 0 || r_len > len as i32 {
            Err(RTTError::FileReadFailed)
        } else {
            unsafe {
                temp.set_len(r_len as usize);
            }
            Ok(temp)
        };
    }

    pub fn delete(path: &str) -> Result<(), RTTError> {
        unsafe {
            let name: CString = path.into();
            if 0 != rttbase::unlink(name.into()) {
                Err(RTTError::FileNotExist)
            } else {
                Ok(())
            }
        }
    }

    pub fn rename(old_path_name: &str, new_path_name: &str) -> Result<(), RTTError> {
        unsafe {
            let old: CString = old_path_name.into();
            let new: CString = new_path_name.into();
            if 0 != rttbase::rename(old.into(), new.into()) {
                Err(RTTError::FileReNameFailed)
            } else {
                Ok(())
            }
        }
    }

    pub fn sync(&self) -> Result<(), RTTError> {
        unsafe {
            if 0 != rttbase::fsync(self.fd) {
                Err(RTTError::FileWriteFailed)
            } else {
                Ok(())
            }
        }
    }
}

pub struct OpenOptions {
    _write: bool,
    _read: bool,
    _append: bool,
    _truncate: bool,
    _create: bool,
    _create_new: bool,
}

impl Default for OpenOptions {
    fn default() -> Self {
        OpenOptions {
            _write: false,
            _read: false,
            _append: false,
            _truncate: false,
            _create: false,
            _create_new: false,
        }
    }
}

impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions {
            ..Default::default()
        }
    }

    pub fn write(&mut self, f: bool) -> &mut Self {
        self._write = f;
        self
    }

    pub fn read(&mut self, f: bool) -> &mut Self {
        self._read = f;
        self
    }

    pub fn append(&mut self, f: bool) -> &mut Self {
        self._append = f;
        self
    }

    pub fn truncate(&mut self, f: bool) -> &mut Self {
        self._truncate = f;
        self
    }

    pub fn create(&mut self, f: bool) -> &mut Self {
        self._create = f;
        self
    }

    pub fn create_new(&mut self, f: bool) -> &mut Self {
        self._create_new = f;
        self
    }

    pub fn open(self, path: &str) -> Result<File, RTTError> {
        if self._create_new {
            let temp = File::raw_open(path, 0);
            if let Ok(_) = temp {
                return Err(RTTError::FileExist);
            }
        }
        let mut flag;
        if self._read && !self._write {
            flag = 0;
        } else if self._write && !self._read {
            flag = 1;
        } else if self._read && self._write {
            flag = 2
        } else {
            return Err(RTTError::FileOpenFailed);
        }

        if self._create_new || self._create {
            flag |= CREATE;
        }

        if self._truncate {
            flag |= TRUNCATION;
        }

        if self._append {
            flag |= APPEND;
        }

        File::raw_open(path, flag)
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe {
            rttbase::close(self.fd);
        }
    }
}
