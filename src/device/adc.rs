use crate::alloc::fmt;
use crate::base::{CVoid, RTBaseError, RTTError};
use crate::device::common::rttbase_device_find;
use crate::device::common::RtDevice;

extern "C" {
    fn rt_adc_enable(dev: *const CVoid, channel: u32) -> RTBaseError;
    fn rt_adc_disable(dev: *const CVoid, channel: u32) -> RTBaseError;
    fn rt_adc_read(dev: *const CVoid, channel: u32) -> u32;
}

struct ADC {
    handle: RtDevice,
    channel: Option<u32>,
}

impl ADC {
    fn new_open(name: &str) -> Result<ADC, RTTError> {
        let h = rttbase_device_find(name)?;
        Ok(ADC {
            handle: h,
            channel: None,
        })
    }

    fn enable_channel(&mut self, _channel: u32) -> Result<&ADC, RTTError> {
        if self.channel == None {
            self.channel = Some(_channel);
        } else {
            return Err(RTTError::DeviceOpenFailed);
        }
        unsafe {
            rt_adc_enable(self.handle.raw(), _channel);
            Ok(self)
        }
    }

    fn read(&self) -> Result<u32, RTTError> {
        unsafe {
            if let Some(ch) = self.channel {
                Ok(rt_adc_read(self.handle.raw(), ch))
            } else {
                Err(RTTError::DeviceReadFailed)
            }
        }
    }
}

impl Drop for ADC {
    fn drop(&mut self) {
        if let Some(ch) = self.channel {
            unsafe {
                rt_adc_disable(self.handle.raw(), ch);
            }
        }
    }
}

impl Clone for ADC {
    fn clone(&self) -> Self {
        ADC {
            handle: self.handle,
            channel: None,
        }
    }
}

impl fmt::Display for ADC {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ch) = self.channel {
            write!(f, "ADC handle: {}, Channel: {}", self.handle, ch)
        } else {
            write!(f, "ADC handle: {}, Channel unbound", self.handle)
        }
    }
}
