//! Passing information between threads

use crate::base::*;
use crate::base::{CVoid, RTTError};
use crate::Box;
use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::mem;

#[inline]
pub(crate) fn rttbase_queue_create(len: usize, message_size: usize) -> *const CVoid {
    let s = CString::new("rust");
    unsafe { rt_mq_create(s.str.as_ptr(), message_size, len, 0) }
}

#[inline]
pub(crate) fn rttbase_queue_send_wait(
    handle: *const CVoid,
    msg: *const CVoid,
    msg_size: usize,
    tick: i32,
) -> isize {
    unsafe { rt_mq_send_wait(handle, msg, msg_size, tick) }
}

#[inline]
pub(crate) fn rttbase_queue_receive(
    handle: *const CVoid,
    msg: *mut CVoid,
    msg_size: usize,
    tick: i32,
) -> isize {
    unsafe { rt_mq_recv(handle, msg, msg_size, tick) }
}

#[inline]
pub(crate) fn rttbase_queue_delete(handle: *const CVoid) {
    unsafe {
        rt_mq_delete(handle);
    }
}

unsafe impl<T> Send for Queue<T> where T: Send {}
unsafe impl<T> Sync for Queue<T> where T: Send {}

#[derive(Debug)]
pub struct Queue<T> {
    queue: *const CVoid,
    /* only for store item type */
    item_type: PhantomData<UnsafeCell<Box<T>>>,
}

impl<T> Queue<T> {
    pub fn new(max_size: usize) -> Result<Queue<T>, RTTError> {
        let handle = rttbase_queue_create(max_size, Self::mem_size());
        if handle == 0 as *const _ {
            return Err(RTTError::OutOfMemory);
        }
        Ok(Queue {
            queue: handle,
            item_type: PhantomData,
        })
    }

    #[inline]
    pub const fn mem_size() -> usize {
        mem::size_of::<*mut T>()
    }

    pub fn send(&self, item: T) -> Result<(), RTTError> {
        Self::send_wait(&self, item, 0)
    }

    pub fn send_wait(&self, item: T, max_wait: i32) -> Result<(), RTTError> {
        let s = Box::new(item);
        let s = Box::into_raw(s);
        let r = if rttbase_queue_send_wait(
            self.queue,
            &s as *const _ as *const CVoid,
            Self::mem_size(),
            max_wait,
        ) != 0
        {
            Err(RTTError::QueueSendTimeout)
        } else {
            Ok(())
        };
        r
    }

    pub fn receive(&self, max_wait: i32) -> Result<T, RTTError> {
        let mut ptr = 0 as *mut T;
        let r = rttbase_queue_receive(
            self.queue,
            &mut ptr as *mut _ as *mut CVoid,
            Self::mem_size(),
            max_wait,
        );
        return if r == 0 {
            Ok(unsafe {
                let y = Box::from_raw(ptr);
                let r = *y;
                r
            })
        } else {
            Err(RTTError::QueueReceiveTimeout)
        };
    }
}

impl<T> Drop for Queue<T> {
    fn drop(&mut self) {
        rttbase_queue_delete(self.queue);
    }
}
