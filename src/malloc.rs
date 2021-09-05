//! The basic memory function uses rtthread API

use crate::base::*;
use core::alloc::{GlobalAlloc, Layout};

#[alloc_error_handler]
fn foo(_: core::alloc::Layout) -> ! {
    panic!("OOM!");
}

pub struct RttAlloc;

unsafe impl GlobalAlloc for RttAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        rt_malloc(layout.size() as usize) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        rt_free(ptr as *mut CVoid)
    }
}
