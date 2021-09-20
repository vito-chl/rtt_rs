use core::marker::PhantomData;
use core::mem;
use core::ptr::NonNull;

use super::raw;

/// Token to spawn a newly-created task in an executor.
///
/// When calling a task function (like `#[embassy::task] async fn my_task() { ... }`), the returned
/// value is a `SpawnToken` that represents an instance of the task, ready to spawn. You must
/// then spawn it into an executor, typically with [`Spawner::spawn()`].
///
/// # Panics
///
/// Dropping a SpawnToken instance panics. You may not "abort" spawning a task in this way.
/// Once you've invoked a task function and obtained a SpawnToken, you *must* spawn it.
#[must_use = "Calling a task function does nothing on its own. You must spawn the returned SpawnToken, typically with Spawner::spawn()"]
pub struct SpawnToken<F> {
    raw_task: Option<NonNull<raw::TaskHeader>>,
    phantom: PhantomData<*mut F>,
}

impl<F> SpawnToken<F> {
    pub(crate) unsafe fn new(raw_task: NonNull<raw::TaskHeader>) -> Self {
        Self {
            raw_task: Some(raw_task),
            phantom: PhantomData,
        }
    }

    pub(crate) fn new_failed() -> Self {
        Self {
            raw_task: None,
            phantom: PhantomData,
        }
    }
}

impl<F> Drop for SpawnToken<F> {
    fn drop(&mut self) {
        // TODO deallocate the task instead.
        panic!("SpawnToken instances may not be dropped. You must pass them to Spawner::spawn()")
    }
}

/// Error returned when spawning a task.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SpawnError {
    /// Too many instances of this task are already running.
    ///
    /// By default, a task marked with `#[embassy::task]` can only have one instance
    /// running at a time. You may allow multiple instances to run in parallel with
    /// `#[embassy::task(pool_size = 4)]`, at the cost of higher RAM usage.
    Busy,
}

/// Handle to spawn tasks into an executor.
///
/// This Spawner can spawn any task (Send and non-Send ones), but it can
/// only be used in the executor thread (it is not Send itself).
///
/// If you want to spawn tasks from another thread, use [SendSpawner].
#[derive(Copy, Clone)]
pub struct Spawner {
    executor: &'static raw::Executor,
    not_send: PhantomData<*mut ()>,
}

impl Spawner {
    pub(crate) fn new(executor: &'static raw::Executor) -> Self {
        Self {
            executor,
            not_send: PhantomData,
        }
    }

    /// Spawn a task into an executor.
    ///
    /// You obtain the `token` by calling a task function (i.e. one marked with `#[embassy::task]).
    pub fn spawn<F>(&self, token: SpawnToken<F>) -> Result<(), SpawnError> {
        let task = token.raw_task;
        mem::forget(token);

        match task {
            Some(task) => {
                unsafe { self.executor.spawn(task) };
                Ok(())
            }
            None => Err(SpawnError::Busy),
        }
    }

    /// Used by the `embassy_macros::main!` macro to throw an error when spawn
    /// fails. This is here to allow conditional use of `defmt::unwrap!`
    /// without introducing a `defmt` feature in the `embassy_macros` package,
    /// which would require use of `-Z namespaced-features`.
    pub fn must_spawn<F>(&self, token: SpawnToken<F>) -> () {
        self.spawn(token).unwrap();
    }
}
