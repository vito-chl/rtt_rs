#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use crate::alloc::fmt::Formatter;
use crate::base::{CString, CVoid, RTTError};
use crate::fmt;
use crate::string::String;
use crate::vec::Vec;

mod rttbase {
    extern "C" {
        pub(crate) fn mkdir(path: *const u8, mode: u32) -> i32;
    }
}

struct DIR(*mut CVoid);

struct Dirent(*mut CVoid);

impl DIR {
    fn make_dir(path: &str) -> Result<(), RTTError> {
        unimplemented!()
    }

    fn remove_dir(path: &str) -> Result<(), RTTError> {
        unimplemented!()
    }

    fn open_dir(path: &str) -> Result<DIR, RTTError> {
        unimplemented!()
    }

    fn close_dir(d: DIR) -> Result<(), RTTError> {
        unimplemented!()
    }

    fn read_dir(d: DIR) -> Result<Dirent, RTTError> {
        unimplemented!()
    }

    fn tell_dir(d: DIR) -> isize {
        unimplemented!()
    }

    fn seek_dir(d: DIR, offset: isize) {
        unimplemented!()
    }

    fn rewind_dir(d: DIR) {
        unimplemented!()
    }
}
