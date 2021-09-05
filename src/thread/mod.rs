//! Thread support using rt-thread API
//!
//! # Example
//! ```
//! use rtt_rs::thread::Thread;
//! for i in 0..2 {
//!    let th = Thread::new().name("th").stack_size(4096).start(move ||{
//! 		for _ in 0..10
//!			{
//! 			print!("hello thread");
//! 			Thread::delay(100);
//!  	    }
//!    });
//! }
//! ```
//!

use crate::alloc::boxed::Box;
use crate::base::*;
use alloc::string::String;
use core::mem;

#[inline]
pub(crate) fn rttbase_thread_mdelay(ms: i32) {
    unsafe {
        rt_thread_mdelay(ms);
    }
}

#[inline]
pub(crate) fn rttbase_thread_delay(tick: u32) {
    #[cfg(not(feature = "rt-smart"))]
    unsafe {
        rt_thread_delay(tick);
    }

    #[cfg(feature = "rt-smart")]
    unimplemented!()
}

#[inline]
pub(crate) fn rttbase_thread_create(
    name: &str,
    func: extern "C" fn(p: *mut CVoid),
    param: *mut CVoid,
    stack_size: u32,
    priority: u8,
    tick: u32,
) -> *const CVoid {
    let s = CString::new(name);
    unsafe { rt_thread_create(s.str.as_ptr(), func, param, stack_size, priority, tick) }
}

#[inline]
pub(crate) fn rttbase_thread_delete(handle: *const CVoid) {
    unsafe {
        rt_thread_delete(handle);
    }
}

#[inline]
pub(crate) fn rttbase_thread_startup(th: *const CVoid) -> isize {
    unsafe { rt_thread_startup(th) }
}

#[inline]
pub(crate) fn rttbase_thread_yield() {
    unsafe {
        rt_thread_yield();
    }
}

#[derive(Debug)]
pub struct Thread(*const CVoid);

impl Thread {
    /// Delay some ticks: ticks
    /// The clock cycle will depend on the configuration of the system
    pub fn delay(tick: u32) {
        rttbase_thread_delay(tick);
    }

    /// Delay some millisecond
    pub fn mdelay(ms: i32) {
        rttbase_thread_mdelay(ms);
    }

    pub fn new() -> ThreadBuilder {
        ThreadBuilder {
            th_name: "uname".into(),
            th_stack_size: 4096,
            th_priority: 10,
            th_ticks: 10,
        }
    }

    pub fn _yield() {
        rttbase_thread_yield();
    }

    /// # Note
    /// The system has the function of automatically reclaiming the end thread.
    /// If a thread is very short, it is likely to end before you do delete.
    /// At this time, the handle is invalid.
    /// If you try to delete it, an assertion will be generated.
    /// So make sure that the thread you want to delete is not finished.
    /// That's why the drop function is not implemented to delete threads.
    pub fn delete_thread(th: Self) {
        rttbase_thread_delete(th.0);
    }

    /// # Note
    /// Please read the `Note` of `fn delete_thread`
    pub fn delete(&self) {
        rttbase_thread_delete(self.0);
    }

    unsafe fn spawn_inner(
        name: String,
        stack_size: u32,
        priority: u8,
        ticks: u32,
        func: Box<dyn FnOnce()>,
    ) -> Result<Self, RTTError> {
        let func = Box::new(func);
        let param = &*func as *const _ as *mut _;

        extern "C" fn thread_func(param: *mut CVoid) {
            unsafe {
                let run = Box::from_raw(param as *mut Box<dyn FnOnce()>);
                run();
            }
        }

        let th_handle = rttbase_thread_create(
            name.as_ref(),
            thread_func,
            param,
            stack_size,
            priority,
            ticks,
        );

        if th_handle == 0 as *const CVoid {
            return Err(RTTError::OutOfMemory);
        }

        let ret = match Self::_startup(th_handle) {
            Ok(_) => {
                mem::forget(func);
                Ok(Thread(th_handle))
            }
            Err(e) => Err(e),
        };

        return ret;
    }

    fn _startup(th: *const CVoid) -> Result<(), RTTError> {
        let ret = rttbase_thread_startup(th);
        return if ret != 0 {
            Err(RTTError::ThreadStartupErr)
        } else {
            Ok(())
        };
    }

    pub fn spawn<F>(name: String, stack_size: u32, priority: u8, ticks: u32, func: F) -> Thread
    where
        F: FnOnce() -> () + Send + 'static,
    {
        unsafe {
            return Self::spawn_inner(name, stack_size, priority, ticks, Box::new(func)).unwrap();
        }
    }
}

unsafe impl Send for Thread {}

pub struct ThreadBuilder {
    th_name: String,
    th_stack_size: u32,
    th_priority: u8,
    th_ticks: u32,
}

impl ThreadBuilder {
    pub fn name(&mut self, name: &str) -> &mut Self {
        self.th_name = name.into();
        self
    }

    pub fn stack_size(&mut self, stack_size: u32) -> &mut Self {
        self.th_stack_size = stack_size;
        self
    }

    pub fn priority(&mut self, priority: u8) -> &mut Self {
        self.th_priority = priority;
        self
    }

    pub fn ticks(&mut self, ticks: u32) -> &mut Self {
        self.th_ticks = ticks;
        self
    }

    pub fn start<F>(&self, func: F) -> Result<Thread, RTTError>
    where
        F: FnOnce() -> (),
        F: Send + 'static,
    {
        Ok(Thread::spawn(
            self.th_name.clone(),
            self.th_stack_size,
            self.th_priority,
            self.th_ticks,
            func,
        ))
    }
}
