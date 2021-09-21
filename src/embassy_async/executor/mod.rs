//! Async task executor.
mod raw;
mod spawner;

use core::marker::PhantomData;

use crate::base::CVoid;
use crate::embassy_async::executor::raw::{task_from_waker, wake_task};
use crate::raw_api::{rt_thread_resume, rt_thread_self, rt_thread_suspend};
use core::task::Waker;
pub use spawner::*;
pub use raw::*;

pub struct Executor {
    inner: raw::Executor,
    thread: *mut CVoid,
    not_send: PhantomData<*mut ()>,
}

fn wake_thread(th: *mut CVoid) {
    unsafe {
        rt_thread_resume(th as _);
    }
}

pub fn device_wake(c: Waker) {
    unsafe {
        let task = task_from_waker(&c);
        wake_task(task);
    }
}

impl Executor {
    /// Create a new Executor.
    pub fn new() -> Self {
        let th = unsafe { rt_thread_self() };
        Self {
            inner: raw::Executor::new(|th| wake_thread(th as _), th as _),
            thread: th as _,
            not_send: PhantomData,
        }
    }

    /// Run the executor.
    ///
    /// The `init` closure is called with a [`Spawner`] that spawns tasks on
    /// this executor. Use it to spawn the initial task(s). After `init` returns,
    /// the executor starts running the tasks.
    ///
    /// To spawn more tasks later, you may keep copies of the [`Spawner`] (it is `Copy`),
    /// for example by passing it as an argument to the initial tasks.
    ///
    /// This function requires `&'static mut self`. This means you have to store the
    /// Executor instance in a place where it'll live forever and grants you mutable
    /// access. There's a few ways to do this:
    ///
    /// - a [Forever](crate::util::Forever) (safe)
    /// - a `static mut` (unsafe)
    /// - a local variable in a function you know never returns (like `fn main() -> !`), upgrading its lifetime with `transmute`. (unsafe)
    ///
    /// This function never returns.
    pub fn run(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
        init(self.inner.spawner());

        loop {
            unsafe {
                self.inner.poll();
                rt_thread_suspend(self.thread as _);
            };
        }
    }
}
