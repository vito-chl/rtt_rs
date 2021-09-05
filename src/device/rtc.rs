//! Time!
//! # Example
//! ```
//! use::rtt_rs::device::rtc;
//!
//! rtc::Time::set_date(2021, 1, 1).unwrap();
//! rtc::Time::set_time(12, 0, 0).unwrap();
//!
//! println!("time: {}", rtc::Time::now());
//! ```

use crate::alloc::string::String;
use crate::base::{CStr, RTBaseError, RTTError};
use crate::fmt;
use crate::fmt::Display;

type TimeType = isize;

extern "C" {
    fn set_date(year: u32, month: u32, day: u32) -> RTBaseError;
    fn set_time(hour: u32, minute: u32, second: u32) -> RTBaseError;
    fn time(t: *const TimeType) -> TimeType;
    fn ctime(time: TimeType) -> *const u8;
}

pub struct Time(TimeType);

impl Time {
    pub fn set_date(year: u32, month: u32, day: u32) -> Result<(), RTTError> {
        unsafe {
            if 0 != set_date(year, month, day) {
                Err(RTTError::DeviceWriteFailed)
            } else {
                Ok(())
            }
        }
    }

    pub fn set_time(hour: u32, minute: u32, second: u32) -> Result<(), RTTError> {
        unsafe {
            if 0 != set_time(hour, minute, second) {
                Err(RTTError::DeviceWriteFailed)
            } else {
                Ok(())
            }
        }
    }

    pub fn now() -> Time {
        Time(Self::now_num())
    }

    pub fn now_num() -> TimeType {
        unsafe { time(0 as *const TimeType) }
    }

    pub fn now_str() -> String {
        unsafe {
            let time_str = ctime(Self::now().0);
            let temp: CStr = time_str.into();
            temp.into()
        }
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = Self::now_str();
        write!(f, "{}", s)
    }
}
