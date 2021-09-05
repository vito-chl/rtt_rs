use crate::alloc::fmt::Formatter;
use crate::base::{CVoid, RTBaseError, RTTError};
use crate::device::common::rttbase_device_find;
use crate::device::common::RtDevice;
use crate::fmt;

extern "C" {
    fn rt_pwm_set(dev: *const CVoid, channel: i32, period: u32, pulse: u32) -> RTBaseError;
    fn rt_pwm_enable(dev: *const CVoid, channel: i32) -> RTBaseError;
    fn rt_pwm_disable(dev: *const CVoid, channel: i32) -> RTBaseError;
}

struct PWM {
    handle: RtDevice,
}

struct PWMChannel<'a> {
    p: &'a PWM,
    ch: i32,
}

impl PWM {
    fn new_open(name: &str) -> Result<PWM, RTTError> {
        let h = rttbase_device_find(name)?;
        Ok(PWM { handle: h })
    }

    fn channel(&self, _channel: i32, period: u32, pulse: u32) -> Result<PWMChannel, RTTError> {
        unsafe {
            if 0 == rt_pwm_set(self.handle.raw(), _channel, period, pulse) {
                Ok(PWMChannel {
                    p: self,
                    ch: _channel,
                })
            } else {
                Err(RTTError::DeviceOpenFailed)
            }
        }
    }
}

impl<'a> fmt::Display for PWMChannel<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "device: {}, channel: {}", self.p.handle, self.ch)
    }
}

impl<'a> PWMChannel<'a> {
    fn enable(&self) -> Result<(), RTTError> {
        unsafe {
            if 0 != rt_pwm_enable(self.p.handle.raw(), self.ch) {
                Err(RTTError::DeviceOpenFailed)
            } else {
                Ok(())
            }
        }
    }
}

impl<'a> Drop for PWMChannel<'a> {
    fn drop(&mut self) {
        unsafe {
            rt_pwm_disable(self.p.handle.raw(), self.ch);
        }
    }
}
