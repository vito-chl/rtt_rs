//! Provide the output function of debugging serial port

#![allow(unused_imports)]

use crate::base::{CString, CVoid};

#[cfg(not(feature = "rt-smart"))]
use crate::base::*;

#[cfg(feature = "rt-smart")]
use crate::base::*;

#[cfg(not(feature = "rt-smart"))]
static mut PRINT_LOCK: *const CVoid = 0 as *const CVoid;

#[cfg(not(feature = "rt-smart"))]
pub fn rttbase_init() {
    let name = CString::new("prt");
    unsafe {
        PRINT_LOCK = rt_mutex_create(name.str.as_ptr(), 1);
    }
}

#[inline]
pub(crate) fn rttbase_print_lock() {
    #[cfg(not(feature = "rt-smart"))]
    unsafe {
        rt_mutex_take(PRINT_LOCK, -1);
    }
}

#[inline]
pub(crate) fn rttbase_print_unlock() {
    #[cfg(not(feature = "rt-smart"))]
    unsafe {
        rt_mutex_release(PRINT_LOCK);
    }
}

#[inline]
pub(crate) fn rttbase_print(str: &str) {
    #[cfg(not(feature = "rt-smart"))]
    let s = CString::new(str);

    #[cfg(not(feature = "rt-smart"))]
    unsafe {
        rt_kputs(s.str.as_ptr());
    }

    #[cfg(feature = "rt-smart")]
    unsafe {
        // puts(s.str.as_ptr());
        for i in str.chars() {
            putchar(i as u32);
        }
    }
}

use core::fmt::{self, Write};

struct StdOut;

impl fmt::Write for StdOut {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        rttbase_print(s);
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    rttbase_print_lock();
    StdOut.write_fmt(args).unwrap();
    rttbase_print_unlock();
}

pub fn _print_unlock(args: fmt::Arguments) {
    StdOut.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! safe_print {
    ($($arg:tt)*) => ({
        $crate::out::_print(format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::out::_print_unlock(format_args!($($arg)*));
    });
}

#[macro_export]
#[allow_internal_unstable(print_internals, format_args_nl)]
macro_rules! println {
    ($($arg:tt)*) => ({
        $crate::out::_print_unlock(format_args_nl!($($arg)*));
    });
}

#[panic_handler]
#[inline(never)]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("{:}", info);
    loop {}
}

pub use core::file;
pub use core::line;
pub use core::stringify;

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! dlog {
    ($($arg:tt)*) => ({
        $crate::println!("[$DBG][{}:{}] {}",
        $crate::out::file!(), $crate::out::line!(), format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => ({
        $crate::println!("[$LOG][{}:{}] {}",
        $crate::out::file!(), $crate::out::line!(), format_args!($($arg)*));
    });
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! dbg {
    () => {
        $crate::println!("[{}:{}]", $crate::out::file!(), $crate::out::line!());
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                $crate::println!("[{}:{}] {} = {:#?}",
                $crate::out::file!(), $crate::out::line!(), $crate::out::stringify!($val), &tmp);
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! dbg {
    () => {};
    ($val:expr $(,)?) => {};
    ($($val:expr),+ $(,)?) => {};
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! dlog {
    ($($arg:tt)*) => {}
}