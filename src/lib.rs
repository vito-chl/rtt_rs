//! RT-THREAD & RUST
//!
//! A simple and easy-to-use system support library
//! that provides basic functions and FS, NET and DEVICE.
//!
//! You can use this library on embedded devices that support rt-thread
//!
//! You can specify a function for “ C ” to call
//!
//! # Example
//! ```
//! #![no_std]
//!
//! use rtt_rs::*;
//! entry!(main);
//! fn main() { /*.....*/ }
//! ```

#![no_std]
#![feature(alloc_error_handler)]
#![feature(allow_internal_unstable)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_fn_trait_bound)]
#![cfg(not(test))]
#![allow(dead_code)]

#[doc = "alloc by rtthread"]
#[global_allocator]
static GLOBAL: malloc::RttAlloc = malloc::RttAlloc;

pub extern crate alloc;

pub mod base;
pub mod malloc;
pub mod mutex;
pub mod out;
pub mod queue;
pub mod raw_api;
pub mod semaphore;
pub mod timer;

pub mod thread;

/// Default is using device
/// if you don't want to use it
/// `rtt_rs={ version = "x.x.x", default-features = false, features = [] }`
#[cfg(feature = "device")]
pub mod device;

#[cfg(feature = "fs")]
pub mod fs;

#[cfg(feature = "net")]
pub mod net;

mod embassy_async;
mod prelude;

pub use prelude::v1::*;

/// This macro is used to indicate the entry function of the system
///
/// # Example
/// ```
/// use rtt_rs::*;
///
/// /* rtthread will call function main */
/// entry!(main);
///
/// fn main(){
///     /* ..... */
/// }
/// ```
///
#[cfg(not(feature = "rt-smart"))]
#[macro_export]
macro_rules! entry {
    ($func: ident) => {
        #[no_mangle]
        pub extern "C" fn rust_main() -> usize {
            $func();
            0
        }
    };
}
