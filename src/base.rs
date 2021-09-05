//! Some basic and fragmented components
//! Provide c function interface

/* from stdlib */
use crate::string::String;
use crate::vec::Vec;

#[repr(u8)]
pub enum CVoid {
    __Variant1,
    __Variant2,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RTTError {
    ThreadStartupErr,
    MutexTakeTimeout,
    SemaphoreTakeTimeout,
    QueueSendTimeout,
    QueueReceiveTimeout,
    OutOfMemory,

    DeviceNotFound,
    DeviceOpenFailed,
    DeviceCloseFailed,
    DeviceReadFailed,
    DeviceWriteFailed,
    DeviceTransFailed,
    DeviceConfigFailed,
    DeviceSetRxCallBackFailed,
    DeviceSetTxCallBackFailed,

    FileOpenFailed,
    FileWriteFailed,
    FileCloseFailed,
    FileReadFailed,
    FileExist,
    FileNotExist,
    FileReNameFailed,

    FuncUnDefine,
}

/// c语言格式的字符串
/// 存储于堆 可扩展
#[derive(Clone)]
pub struct CString {
    pub str: Vec<u8>,
}

impl CString {
    pub(crate) fn new(str: &str) -> Self {
        let mut temp: Vec<u8> = str.as_bytes().iter().cloned().collect();
        temp.push(0);
        CString { str: temp }
    }
}

impl From<String> for CString {
    fn from(s: String) -> Self {
        CString::new(s.as_str())
    }
}

impl From<&str> for CString {
    fn from(s: &str) -> Self {
        CString::new(s)
    }
}

impl Into<*const u8> for CString {
    fn into(self) -> *const u8 {
        self.str.as_ptr()
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct CStr(*const u8);

impl CStr {
    pub fn new(raw: *const u8) -> CStr {
        CStr(raw)
    }
}

impl From<*const u8> for CStr {
    fn from(raw: *const u8) -> Self {
        CStr::new(raw)
    }
}

impl Into<*const u8> for CStr {
    fn into(self) -> *const u8 {
        self.0
    }
}

impl From<CString> for CStr {
    fn from(s: CString) -> Self {
        CStr(s.str.as_ptr())
    }
}

const STR_MAX_LEN: usize = 128;

impl Into<String> for CStr {
    fn into(self) -> String {
        unsafe {
            let mut ptr = self.0;
            let mut out_str = String::new();
            for _ in 0..STR_MAX_LEN {
                let a = *ptr;
                if a == 0 {
                    break;
                } else {
                    out_str.push(a as char);
                }
                ptr = ptr.offset(1);
            }
            out_str
        }
    }
}

pub type RTBaseError = isize;
pub type CCharPtr = *const u8;
pub type CVoidPtr = *const CVoid;

#[cfg(not(feature = "rt-smart"))]
extern "C" {
    /* For out */
    pub(crate) fn rt_kputs(s: *const u8);

    /* For alloc */
    pub(crate) fn rt_malloc(size: usize) -> *mut u8;
    pub(crate) fn rt_free(ptr: *mut CVoid);

    /* For thread */
    pub(crate) fn rt_thread_create(
        name: *const u8,
        func: extern "C" fn(p: *mut CVoid),
        param: *mut CVoid,
        stack_size: u32,
        priority: u8,
        tick: u32,
    ) -> *const CVoid;
    pub(crate) fn rt_thread_delete(handle: *const CVoid) -> isize;
    pub(crate) fn rt_thread_startup(th: *const CVoid) -> isize;
    pub(crate) fn rt_thread_yield() -> isize;
    pub(crate) fn rt_thread_delay(tick: u32) -> isize;
    pub(crate) fn rt_thread_mdelay(tick: i32) -> isize;

    /* For mutex */
    pub(crate) fn rt_mutex_create(name: *const u8, flag: u8) -> *const CVoid;
    pub(crate) fn rt_mutex_take(handle: *const CVoid, tick: i32) -> isize;
    pub(crate) fn rt_mutex_release(handle: *const CVoid) -> isize;
    pub(crate) fn rt_mutex_delete(handle: *const CVoid) -> isize;

    /* For queue */
    pub(crate) fn rt_mq_create(
        name: *const u8,
        message_size: usize,
        len: usize,
        flag: u8,
    ) -> *const CVoid;
    pub(crate) fn rt_mq_send_wait(
        handle: *const CVoid,
        msg: *const CVoid,
        msg_size: usize,
        tick: i32,
    ) -> isize;
    pub(crate) fn rt_mq_recv(
        handle: *const CVoid,
        msg: *mut CVoid,
        msg_size: usize,
        tick: i32,
    ) -> isize;
    pub(crate) fn rt_mq_delete(handle: *const CVoid) -> isize;

    /* For semaphore */
    pub(crate) fn rt_sem_create(name: *const u8, val: u32, flag: u8) -> *const CVoid;
    pub(crate) fn rt_sem_delete(m: *const CVoid) -> isize;
    pub(crate) fn rt_sem_try_take(m: *const CVoid) -> isize;
    pub(crate) fn rt_sem_take(m: *const CVoid, tick: i32) -> isize;
    pub(crate) fn rt_sem_release(m: *const CVoid) -> isize;
}

#[cfg(feature = "rt-smart")]
#[link(name = "c")]
extern "C" {
    /* For out */
    // pub(crate) fn puts(a: *const u8) -> u32;
    pub(crate) fn putchar(a: u32) -> u32;

    /* For alloc  */
    pub(crate) fn malloc(s: u32) -> *mut CVoid;
    pub(crate) fn free(ptr: *mut CVoid);

    /* For thread */
}

#[cfg(feature = "rt-smart")]
#[link(name = "rts")]
extern "C" {
    /* For alloc  */
    pub(crate) fn rt_malloc(s: usize) -> *mut CVoid;
    pub(crate) fn rt_free(ptr: *mut CVoid);

    /* For proc */
    // TODO: rtthread channel, share memory

    /* For thread */
    pub(crate) fn rt_thread_create(
        name: *const u8,
        func: extern "C" fn(p: *mut CVoid),
        param: *mut CVoid,
        stack_size: u32,
        priority: u8,
        tick: u32,
    ) -> *const CVoid;
    pub(crate) fn rt_thread_delete(handle: *const CVoid) -> isize;
    pub(crate) fn rt_thread_startup(th: *const CVoid) -> isize;
    pub(crate) fn rt_thread_yield() -> isize;
    pub(crate) fn rt_thread_delay(tick: u32) -> isize;
    pub(crate) fn rt_thread_mdelay(tick: i32) -> isize;

    /* For mutex */
    pub(crate) fn rt_mutex_create(name: *const u8, flag: u8) -> *const CVoid;
    pub(crate) fn rt_mutex_take(handle: *const CVoid, tick: i32) -> isize;
    pub(crate) fn rt_mutex_release(handle: *const CVoid) -> isize;
    pub(crate) fn rt_mutex_delete(handle: *const CVoid) -> isize;

    /* For queue */
    pub(crate) fn rt_mq_create(
        name: *const u8,
        message_size: usize,
        len: usize,
        flag: u8,
    ) -> *const CVoid;
    pub(crate) fn rt_mq_send_wait(
        handle: *const CVoid,
        msg: *const CVoid,
        msg_size: usize,
        tick: i32,
    ) -> isize;
    pub(crate) fn rt_mq_recv(
        handle: *const CVoid,
        msg: *mut CVoid,
        msg_size: usize,
        tick: i32,
    ) -> isize;
    pub(crate) fn rt_mq_delete(handle: *const CVoid) -> isize;

    /* For semaphore */
    pub(crate) fn rttbase_semaphore_create() -> *const CVoid;
    pub(crate) fn rttbase_semaphore_delete(m: *const CVoid);
    pub(crate) fn rttbase_semaphore_try_take(m: *const CVoid) -> u32;
    pub(crate) fn rttbase_semaphore_take(m: *const CVoid, tick: i32) -> u32;
    pub(crate) fn rttbase_semaphore_release(m: *const CVoid);
}

#[cfg(feature = "rt-smart")]
#[no_mangle]
pub extern "C" fn __aeabi_unwind_cpp_pr0() {}
