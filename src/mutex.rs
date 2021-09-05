//! Safe PV operation provided by the system using rt-thread API
//!
//! # Example
//! ```
//!
//! use rtt_rs::Arc;
//! use rtt_rs::mutex::Mutex;
//! use rtt_rs::thread::Thread;
//!
//! let num = Arc::new(Mutex::new(5).unwrap());
//!
//! for i in 0..2 {
//!     let counter = Arc::clone(&num);
//!    	let th = Thread::new().name("th").stack_size(8192).start(move ||{
//!    		for _ in 0..10 {
//!    			{
//!    				let mut th_num = counter.lock().unwrap();
//!                 *th_num += 1;
//!    				print!("th{}: {}\n",i,*th_num);
//!    			}
//!    			Thread::mdelay(100);
//!    		}
//!    	});
//! }
//! ```

#![allow(dead_code)]

use crate::base::*;
use alloc::fmt;
pub use alloc::sync::{Arc, Weak};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

#[inline]
pub(crate) fn rttbase_mutex_create() -> *const CVoid {
    let s = CString::new("rust");
    unsafe { rt_mutex_create(s.str.as_ptr(), 1) }
}

#[inline]
pub(crate) fn rttbase_mutex_delete(handle: *const CVoid) {
    unsafe {
        rt_mutex_delete(handle);
    }
}

#[inline]
pub(crate) fn rttbase_mutex_take(handle: *const CVoid, tick: i32) -> isize {
    unsafe { rt_mutex_take(handle, tick) }
}

#[inline]
pub(crate) fn rttbase_mutex_release(handle: *const CVoid) {
    unsafe {
        rt_mutex_release(handle);
    }
}

const RT_WAITING_FOREVER: i32 = -1;

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

pub struct Mutex<T: ?Sized> {
    mutex: MutexRaw,
    data: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    pub fn new(t: T) -> Result<Self, RTTError> {
        Ok(Mutex {
            mutex: MutexRaw::create()?,
            data: UnsafeCell::new(t),
        })
    }

    pub fn try_lock(&self, max_wait: i32) -> Result<MutexGuard<T>, RTTError> {
        self.mutex.take(max_wait)?;
        Ok(MutexGuard {
            __mutex: &self.mutex,
            __data: &self.data,
        })
    }

    pub fn lock(&self) -> Result<MutexGuard<T>, RTTError> {
        self.mutex.take(RT_WAITING_FOREVER)?;
        Ok(MutexGuard {
            __mutex: &self.mutex,
            __data: &self.data,
        })
    }
}

impl<T: ?Sized> fmt::Debug for Mutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Mutex address: {:?}", self.mutex)
    }
}

pub struct MutexGuard<'a, T: ?Sized + 'a> {
    __mutex: &'a MutexRaw,
    __data: &'a UnsafeCell<T>,
}

impl<'mutex, T: ?Sized> Deref for MutexGuard<'mutex, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.__data.get() }
    }
}

impl<'mutex, T: ?Sized> DerefMut for MutexGuard<'mutex, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.__data.get() }
    }
}

impl<'a, T: ?Sized> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.__mutex.release();
    }
}

pub struct MutexRaw(*const CVoid);

impl fmt::Debug for MutexRaw {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl MutexRaw {
    fn create() -> Result<Self, RTTError> {
        let m = rttbase_mutex_create();
        if m == 0 as *const _ {
            return Err(RTTError::OutOfMemory);
        }
        Ok(MutexRaw(m))
    }

    fn take(&self, max_wait: i32) -> Result<(), RTTError> {
        let ret = rttbase_mutex_take(self.0, max_wait);
        if ret != 0 {
            return Err(RTTError::MutexTakeTimeout);
        }

        Ok(())
    }

    fn release(&self) {
        rttbase_mutex_release(self.0);
    }

    fn drop(&mut self) {
        rttbase_mutex_delete(self.0);
    }
}
