//! A lightweight kernel object used to solve the synchronization problem between threads
//!
//! # Example
//! ```
//! use rtt_rs::semaphore;
//! use rtt_rs::Arc;
//! use rtt_rs::thread::Thread;
//!
//! let g_sem = Arc::new(semaphore::Semaphore::new().unwrap());
//!
//! let th1_sem = g_sem.clone();
//! let th = Thread::new().name("th").stack_size(8192).start(move ||{
//!     for _ in 0..10 {
//!         th1_sem.release();
//!    	    Thread::mdelay(100);
//!     }
//! });
//!
//! let th2_sem = g_sem.clone();
//! let th = Thread::new().name(rec).stack_size(8192).start(move ||{
//!     loop{
//!         th2_sem.take_wait_forever();
//!         print!("Rec Sem");
//!     }
//! });
//! ```

#![allow(dead_code)]

use crate::base::{
    rt_sem_create, rt_sem_delete, rt_sem_release, rt_sem_take, rt_sem_try_take, CString, CVoid,
    RTTError,
};
use core::cell::UnsafeCell;

#[inline]
pub(crate) fn rttbase_semaphore_create() -> *const CVoid {
    let s = CString::new("rust");
    unsafe { rt_sem_create(s.str.as_ptr(), 0, 0) }
}

#[inline]
pub(crate) fn rttbase_semaphore_try_take(handle: *const CVoid) -> isize {
    unsafe { rt_sem_try_take(handle) }
}

#[inline]
pub(crate) fn rttbase_semaphore_take(handle: *const CVoid, tick: i32) -> isize {
    unsafe { rt_sem_take(handle, tick) }
}

#[inline]
pub(crate) fn rttbase_semaphore_release(handle: *const CVoid) -> isize {
    unsafe { rt_sem_release(handle) }
}

#[inline]
pub(crate) fn rttbase_semaphore_delete(handle: *const CVoid) {
    unsafe {
        let _ = rt_sem_delete(handle);
    }
}

unsafe impl Send for Semaphore {}
unsafe impl Sync for Semaphore {}

pub struct Semaphore(UnsafeCell<*const CVoid>);

impl Semaphore {
    pub fn new() -> Result<Self, RTTError> {
        let m = rttbase_semaphore_create();
        if m == 0 as *const _ {
            return Err(RTTError::OutOfMemory);
        }
        Ok(Semaphore(UnsafeCell::new(m)))
    }

    pub fn try_take(&self) -> Result<(), RTTError> {
        unsafe {
            let m = rttbase_semaphore_try_take(*self.0.get());
            if m != 0 {
                return Err(RTTError::SemaphoreTakeTimeout);
            }
            Ok(())
        }
    }

    pub fn take_wait_forever(&self) -> Result<(), RTTError> {
        let ret = unsafe { rttbase_semaphore_take(*self.0.get(), -1) };

        if ret != 0 {
            return Err(RTTError::SemaphoreTakeTimeout);
        }

        Ok(())
    }

    pub fn take(&self, max_wait: i32) -> Result<(), RTTError> {
        let ret = unsafe { rttbase_semaphore_take(*self.0.get(), max_wait) };

        if ret != 0 {
            return Err(RTTError::SemaphoreTakeTimeout);
        }

        Ok(())
    }

    pub fn release(&self) {
        unsafe {
            rttbase_semaphore_release(*self.0.get());
        }
    }

    fn drop(&mut self) {
        unsafe { rttbase_semaphore_delete(*self.0.get()) }
    }
}
