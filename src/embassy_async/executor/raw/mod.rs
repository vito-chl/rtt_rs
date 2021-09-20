//! Raw executor.
//!
//! This module exposes "raw" Executor and Task structs for more low level control.
//!
//! ## WARNING: here be dragons!
//!
//! Using this module requires respecting subtle safety contracts. If you can, prefer using the safe
//! executor wrappers in [`crate::executor`] and the [`crate::task`] macro, which are fully safe.

mod run_queue;
pub(crate) mod util;
mod waker;

use atomic_polyfill::{AtomicU32, Ordering};
use core::cell::Cell;
use core::future::Future;
use core::pin::Pin;
use core::ptr::NonNull;
use core::task::{Context, Poll};
use core::{mem, ptr};

use self::run_queue::{RunQueue, RunQueueItem};
use self::util::UninitCell;
use super::SpawnToken;

pub use self::waker::task_from_waker;

/// Task is spawned (has a future)
pub(crate) const STATE_SPAWNED: u32 = 1 << 0;
/// Task is in the executor run queue
pub(crate) const STATE_RUN_QUEUED: u32 = 1 << 1;

/// Raw task header for use in task pointers.
///
/// This is an opaque struct, used for raw pointers to tasks, for use
/// with funtions like [`wake_task`] and [`task_from_waker`].
pub struct TaskHeader {
    pub(crate) state: AtomicU32,
    pub(crate) run_queue_item: RunQueueItem,
    pub(crate) executor: Cell<*const Executor>,
    // Valid if state != 0
    pub(crate) poll_fn: UninitCell<unsafe fn(NonNull<TaskHeader>)>, // Valid if STATE_SPAWNED
}

impl TaskHeader {
    pub(crate) const fn new() -> Self {
        Self {
            state: AtomicU32::new(0),
            run_queue_item: RunQueueItem::new(),
            executor: Cell::new(ptr::null()),
            poll_fn: UninitCell::uninit(),
        }
    }

    pub(crate) unsafe fn enqueue(&self) {
        let mut current = self.state.load(Ordering::Acquire);
        loop {
            // If already scheduled, or if not started,
            if (current & STATE_RUN_QUEUED != 0) || (current & STATE_SPAWNED == 0) {
                return;
            }

            // Mark it as scheduled
            let new = current | STATE_RUN_QUEUED;

            match self.state.compare_exchange_weak(
                current,
                new,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(next_current) => current = next_current,
            }
        }

        // We have just marked the task as scheduled, so enqueue it.
        let executor = &*self.executor.get();
        executor.enqueue(self as *const TaskHeader as *mut TaskHeader);
    }
}

/// Raw storage in which a task can be spawned.
///
/// This struct holds the necessary memory to spawn one task whose future is `F`.
/// At a given time, the `Task` may be in spawned or not-spawned state. You may spawn it
/// with [`Task::spawn()`], which will fail if it is already spawned.
///
/// A `TaskStorage` must live forever, it may not be deallocated even after the task has finished
/// running. Hence the relevant methods require `&'static self`. It may be reused, however.
///
/// Internally, the [embassy::task](crate::task) macro allocates an array of `TaskStorage`s
/// in a `static`. The most common reason to use the raw `Task` is to have control of where
/// the memory for the task is allocated: on the stack, or on the heap with e.g. `Box::leak`, etc.

// repr(C) is needed to guarantee that the Task is located at offset 0
// This makes it safe to cast between Task and Task pointers.
#[repr(C)]
pub struct TaskStorage<F: Future + 'static> {
    raw: TaskHeader,
    future: UninitCell<F>, // Valid if STATE_SPAWNED
}

impl<F: Future + 'static> TaskStorage<F> {
    /// Create a new Task, in not-spawned state.
    pub const fn new() -> Self {
        Self {
            raw: TaskHeader::new(),
            future: UninitCell::uninit(),
        }
    }

    /// Try to spawn a task in a pool.
    ///
    /// See [`Self::spawn()`] for details.
    ///
    /// This will loop over the pool and spawn the task in the first storage that
    /// is currently free. If none is free,
    pub fn spawn_pool(pool: &'static [Self], future: impl FnOnce() -> F) -> SpawnToken<F> {
        for task in pool {
            if task.spawn_allocate() {
                return unsafe { task.spawn_initialize(future) };
            }
        }

        SpawnToken::new_failed()
    }

    /// Try to spawn the task.
    ///
    /// The `future` closure constructs the future. It's only called if spawning is
    /// actually possible. It is a closure instead of a simple `future: F` param to ensure
    /// the future is constructed in-place, avoiding a temporary copy in the stack thanks to
    /// NRVO optimizations.
    ///
    /// This function will fail if the task is already spawned and has not finished running.
    /// In this case, the error is delayed: a "poisoned" SpawnToken is returned, which will
    /// cause [`Executor::spawn()`] to return the error.
    ///
    /// Once the task has finished running, you may spawn it again. It is allowed to spawn it
    /// on a different executor.
    pub fn spawn(&'static self, future: impl FnOnce() -> F) -> SpawnToken<F> {
        if self.spawn_allocate() {
            unsafe { self.spawn_initialize(future) }
        } else {
            SpawnToken::new_failed()
        }
    }

    fn spawn_allocate(&'static self) -> bool {
        let state = STATE_SPAWNED | STATE_RUN_QUEUED;
        self.raw
            .state
            .compare_exchange(0, state, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    unsafe fn spawn_initialize(&'static self, future: impl FnOnce() -> F) -> SpawnToken<F> {
        // Initialize the task
        self.raw.poll_fn.write(Self::poll);
        self.future.write(future());

        SpawnToken::new(NonNull::new_unchecked(&self.raw as *const TaskHeader as _))
    }

    unsafe fn poll(p: NonNull<TaskHeader>) {
        let this = &*(p.as_ptr() as *const TaskStorage<F>);

        let future = Pin::new_unchecked(this.future.as_mut());
        let waker = waker::from_task(p);
        let mut cx = Context::from_waker(&waker);
        match future.poll(&mut cx) {
            Poll::Ready(_) => {
                this.future.drop_in_place();
                this.raw.state.fetch_and(!STATE_SPAWNED, Ordering::AcqRel);
            }
            Poll::Pending => {}
        }

        // the compiler is emitting a virtual call for waker drop, but we know
        // it's a noop for our waker.
        mem::forget(waker);
    }
}

// unsafe impl<F: Future + 'static> Sync for TaskStorage<F> {}

/// Raw executor.
///
/// This is the core of the Embassy executor. It is low-level, requiring manual
/// handling of wakeups and task polling. If you can, prefer using one of the
/// higher level executors in [`crate::executor`].
///
/// The raw executor leaves it up to you to handle wakeups and scheduling:
///
/// - To get the executor to do work, call `poll()`. This will poll all queued tasks (all tasks
///   that "want to run").
/// - You must supply a `signal_fn`. The executor will call it to notify you it has work
///   to do. You must arrange for `poll()` to be called as soon as possible.
///
/// `signal_fn` can be called from *any* context: any thread, any interrupt priority
/// level, etc. It may be called synchronously from any `Executor` method call as well.
/// You must deal with this correctly.
///
/// In particular, you must NOT call `poll` directly from `signal_fn`, as this violates
/// the requirement for `poll` to not be called reentrantly.
pub struct Executor {
    run_queue: RunQueue,
    signal_fn: fn(*mut ()),
    signal_ctx: *mut (),
}

impl Executor {
    /// Create a new executor.
    ///
    /// When the executor has work to do, it will call `signal_fn` with
    /// `signal_ctx` as argument.
    ///
    /// See [`Executor`] docs for details on `signal_fn`.
    pub fn new(signal_fn: fn(*mut ()), signal_ctx: *mut ()) -> Self {
        #[cfg(feature = "time")]
            let alarm = unsafe { unwrap!(driver::allocate_alarm()) };
        #[cfg(feature = "time")]
            driver::set_alarm_callback(alarm, signal_fn, signal_ctx);

        Self {
            run_queue: RunQueue::new(),
            signal_fn,
            signal_ctx,

            #[cfg(feature = "time")]
            timer_queue: timer_queue::TimerQueue::new(),
            #[cfg(feature = "time")]
            alarm,
        }
    }

    /// Enqueue a task in the task queue
    ///
    /// # Safety
    /// - `task` must be a valid pointer to a spawned task.
    /// - `task` must be set up to run in this executor.
    /// - `task` must NOT be already enqueued (in this executor or another one).
    unsafe fn enqueue(&self, task: *mut TaskHeader) {
        if self.run_queue.enqueue(task) {
            (self.signal_fn)(self.signal_ctx)
        }
    }

    /// Spawn a task in this executor.
    ///
    /// # Safety
    ///
    /// `task` must be a valid pointer to an initialized but not-already-spawned task.
    ///
    /// It is OK to use `unsafe` to call this from a thread that's not the executor thread.
    /// In this case, the task's Future must be Send. This is because this is effectively
    /// sending the task to the executor thread.
    pub(super) unsafe fn spawn(&'static self, task: NonNull<TaskHeader>) {
        let task = task.as_ref();
        task.executor.set(self);
        self.enqueue(task as *const _ as _);
    }

    /// Poll all queued tasks in this executor.
    ///
    /// This loops over all tasks that are queued to be polled (i.e. they're
    /// freshly spawned or they've been woken). Other tasks are not polled.
    ///
    /// You must call `poll` after receiving a call to `signal_fn`. It is OK
    /// to call `poll` even when not requested by `signal_fn`, but it wastes
    /// energy.
    ///
    /// # Safety
    ///
    /// You must NOT call `poll` reentrantly on the same executor.
    ///
    /// In particular, note that `poll` may call `signal_fn` synchronously. Therefore, you
    /// must NOT directly call `poll()` from your `signal_fn`. Instead, `signal_fn` has to
    /// somehow schedule for `poll()` to be called later, at a time you know for sure there's
    /// no `poll()` already running.
    pub unsafe fn poll(&'static self) {
        #[cfg(feature = "time")]
            self.timer_queue.dequeue_expired(Instant::now(), |p| {
            p.as_ref().enqueue();
        });

        self.run_queue.dequeue_all(|p| {
            let task = p.as_ref();

            #[cfg(feature = "time")]
                task.expires_at.set(Instant::MAX);

            let state = task.state.fetch_and(!STATE_RUN_QUEUED, Ordering::AcqRel);
            if state & STATE_SPAWNED == 0 {
                // If task is not running, ignore it. This can happen in the following scenario:
                //   - Task gets dequeued, poll starts
                //   - While task is being polled, it gets woken. It gets placed in the queue.
                //   - Task poll finishes, returning done=true
                //   - RUNNING bit is cleared, but the task is already in the queue.
                return;
            }

            // Run the task
            task.poll_fn.read()(p as _);
        });
    }

    /// Get a spawner that spawns tasks in this executor.
    ///
    /// It is OK to call this method multiple times to obtain multiple
    /// `Spawner`s. You may also copy `Spawner`s.
    pub fn spawner(&'static self) -> super::Spawner {
        super::Spawner::new(self)
    }
}

/// Wake a task by raw pointer.
///
/// You can obtain task pointers from `Waker`s using [`task_from_waker`].
pub unsafe fn wake_task(task: NonNull<TaskHeader>) {
    task.as_ref().enqueue();
}
