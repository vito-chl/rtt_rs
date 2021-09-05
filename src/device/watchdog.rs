//!
//! # Example:
//!```
//! use crate::rtt_rs::device::watchdog;
//!
//! let t = watchdog::WatchDog::new().timeout(500).open().unwrap().start()?;
//! let d = t.dog();
//!
//! /* in another thread */
//! /* .... */
//! loop {
//!     d.active();
//!     /* do something.... */
//! }
//! /* .... */
//!```

use crate::base::{CVoid, RTBaseError, RTTError};
use crate::device::common::RtDevice;
use crate::device::common::{rttbase_device_close, rttbase_device_ctrl, rttbase_device_find, Open};
use crate::string::ToString;
use alloc::string::String;

extern "C" {
    fn rt_device_init(dev: *const CVoid) -> RTBaseError;
}

pub struct WatchDog {
    handle: RtDevice,
}

impl WatchDog {
    pub fn new() -> WatchDogBuilder {
        WatchDogBuilder {
            name: "wdt".to_string(),
            timeout: 1000,
        }
    }

    pub fn start(&self) -> Result<&Self, RTTError> {
        rttbase_device_ctrl(self.handle.raw(), 5, 0 as _)?;
        Ok(self)
    }

    pub fn stop(&self) -> Result<&Self, RTTError> {
        rttbase_device_ctrl(self.handle.raw(), 6, 0 as _)?;
        Ok(self)
    }

    /// get dog to active
    /// it has the trait : send/sync
    pub fn dog(&self) -> WatchDogGuard {
        WatchDogGuard { 0: self.handle }
    }
}

impl Drop for WatchDog {
    fn drop(&mut self) {
        rttbase_device_close(self.handle.raw()).unwrap();
    }
}

pub struct WatchDogGuard(RtDevice);

unsafe impl Send for WatchDogGuard {}

unsafe impl Sync for WatchDogGuard {}

impl WatchDogGuard {
    pub fn active(&self) -> Result<(), RTTError> {
        rttbase_device_ctrl(self.0.raw(), 3, 0 as _)
    }
}

impl Open<WatchDogBuilder> for WatchDog {
    fn open(builder: &WatchDogBuilder) -> Result<Self, RTTError> {
        let dev;

        dev = rttbase_device_find(builder.name.as_str())?;
        unsafe {
            if 0 != rt_device_init(dev.raw()) {
                return Err(RTTError::DeviceOpenFailed);
            }
        }
        rttbase_device_ctrl(dev.raw(), 2, builder.timeout as *const _)?;

        Ok(WatchDog { handle: dev })
    }
}

pub struct WatchDogBuilder {
    name: String,
    timeout: u32,
}

impl WatchDogBuilder {
    pub fn open(&self) -> Result<WatchDog, RTTError> {
        WatchDog::open(self)
    }

    pub fn name(&mut self, s: &str) -> &mut Self {
        self.name = String::from(s);
        self
    }

    pub fn timeout(&mut self, t: u32) -> &mut Self {
        self.timeout = t;
        self
    }
}
