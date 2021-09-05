use crate::base::{CVoid, RTTError};
use crate::device::common::{rttbase_device_find, RtDevice};
use crate::vec::Vec;

extern "C" {
    fn rt_i2c_transfer(bus: *const CVoid, msg: *const I2CMsg, num: u32) -> usize;
}

#[repr(C)]
struct I2CMsg {
    address: u16,
    flags: u16,
    len: u16,
    buf: *const u8,
}

pub struct I2CDevice {
    bus: RtDevice,
    address: u16,
    add_10bits: bool,
}

// TODO: 增加批量操作功能

impl I2CDevice {
    /// # Note:
    /// address don't include w/r bit
    pub fn new_open(bas_name: &str, add: u16, _add_10bits: bool) -> Result<Self, RTTError> {
        Ok(I2CDevice {
            bus: rttbase_device_find(bas_name)?,
            address: add,
            add_10bits: _add_10bits,
        })
    }

    #[inline]
    fn inner_read(&self, _len: usize, _flag: u16) -> Result<Vec<u8>, RTTError> {
        let mut buf: Vec<u8> = Vec::new();
        buf.resize(_len, 0);

        let msg = I2CMsg {
            address: self.address,
            flags: _flag,
            len: _len as u16,
            buf: buf.as_ptr(),
        };
        unsafe {
            if 1 != rt_i2c_transfer(self.bus.raw() as *const CVoid, &msg as *const I2CMsg, 1) {
                return Err(RTTError::DeviceReadFailed);
            }
        }
        Ok(buf)
    }

    pub fn read_ack(&self, len: usize, ignore_ack: bool) -> Result<Vec<u8>, RTTError> {
        let mut flag = 1 as u16;
        flag |= if ignore_ack { 1 << 5 } else { 0 };
        flag |= if self.add_10bits { 1 << 2 } else { 0 };
        Self::inner_read(self, len, flag)
    }

    pub fn read_nack(&self, len: usize, ignore_ack: bool) -> Result<Vec<u8>, RTTError> {
        let mut flag = 1 as u16;
        flag |= if ignore_ack { 1 << 5 } else { 0 };
        flag |= if self.add_10bits { 1 << 2 } else { 0 };
        flag |= 1 << 6;
        Self::inner_read(self, len, flag)
    }

    pub fn write(&self, buf: Vec<u8>, ignore_ack: bool) -> Result<(), RTTError> {
        let mut flag = 0 as u16;
        flag |= if ignore_ack { 1 << 5 } else { 0 };
        flag |= if self.add_10bits { 1 << 2 } else { 0 };

        let msg = I2CMsg {
            address: self.address,
            flags: flag,
            len: buf.len() as u16,
            buf: buf.as_ptr(),
        };
        unsafe {
            if 1 != rt_i2c_transfer(self.bus.raw() as *const CVoid, &msg as *const I2CMsg, 1) {
                return Err(RTTError::DeviceWriteFailed);
            }
        }
        Ok(())
    }
}
