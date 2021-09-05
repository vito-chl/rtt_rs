use crate::alloc::fmt;
use crate::base::{CVoid, RTBaseError, RTTError};
use crate::device::common::rttbase_device_find;
use crate::device::common::RtDevice;

extern "C" {
    fn rt_dac_enable(dev: *const CVoid, channel: u32) -> RTBaseError;
    fn rt_dac_disable(dev: *const CVoid, channel: u32) -> RTBaseError;
    fn rt_dac_write(dev: *const CVoid, val: u32, channel: u32) -> RTBaseError;
}

struct DAC {
    handle: RtDevice,
    channel: Option<u32>,
}

impl DAC {
    fn new_open(name: &str) -> Result<DAC, RTTError> {
        let h = rttbase_device_find(name)?;
        Ok(DAC {
            handle: h,
            channel: None,
        })
    }

    fn enable_channel(&mut self, _channel: u32) -> Result<&DAC, RTTError> {
        if self.channel == None {
            self.channel = Some(_channel);
        } else {
            return Err(RTTError::DeviceOpenFailed);
        }
        unsafe {
            rt_dac_enable(self.handle.raw(), _channel);
            Ok(self)
        }
    }

    fn write(&self, val: u32) -> Result<(), RTTError> {
        unsafe {
            if let Some(ch) = self.channel {
                if 0 != rt_dac_write(self.handle.raw(), val, ch) {
                    Err(RTTError::DeviceWriteFailed)
                } else {
                    Ok(())
                }
            } else {
                Err(RTTError::DeviceWriteFailed)
            }
        }
    }
}

impl Drop for DAC {
    fn drop(&mut self) {
        if let Some(ch) = self.channel {
            unsafe {
                rt_dac_disable(self.handle.raw(), ch);
            }
        }
    }
}

impl Clone for DAC {
    fn clone(&self) -> Self {
        DAC {
            handle: self.handle,
            channel: None,
        }
    }
}

impl fmt::Display for DAC {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ch) = self.channel {
            write!(f, "DAC handle: {}, Channel: {}", self.handle, ch)
        } else {
            write!(f, "DAC handle: {}, Channel unbound", self.handle)
        }
    }
}
