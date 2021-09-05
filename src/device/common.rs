use crate::alloc::fmt::Formatter;
use crate::base::RTTError;
use crate::base::{CString, CVoid, RTBaseError};
use crate::fmt;

type RTBaseDevice = *const CVoid;
const RTEOK: RTBaseError = 0;

extern "C" {
    fn rt_device_find(name: *const u8) -> RTBaseDevice;
    fn rt_device_open(dev: RTBaseDevice, open_flag: u16) -> RTBaseError;
    fn rt_device_read(dev: RTBaseDevice, pos: isize, buf: *mut CVoid, size: usize) -> usize;
    fn rt_device_write(dev: RTBaseDevice, pos: isize, buf: *const CVoid, size: usize) -> usize;
    fn rt_device_control(dev: RTBaseDevice, cmd: i32, arg: *const CVoid) -> RTBaseError;

    pub(crate) fn rt_device_set_rx_indicate(
        dev: RTBaseDevice,
        ind: extern "C" fn(dev: RTBaseDevice, size: usize) -> RTBaseError,
    ) -> RTBaseError;

    pub(crate) fn rt_device_set_tx_complete(
        dev: RTBaseDevice,
        ind: extern "C" fn(dev: RTBaseDevice, buf: *const CVoid) -> RTBaseError,
    ) -> RTBaseError;

    pub(crate) fn rt_device_close(dev: RTBaseDevice) -> RTBaseError;
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct RtDevice(pub RTBaseDevice);

impl fmt::Display for RtDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl RtDevice {
    pub(crate) fn raw(&self) -> *const CVoid {
        return self.0;
    }
}

impl From<*const CVoid> for RtDevice {
    fn from(n: *const CVoid) -> Self {
        RtDevice(n)
    }
}

pub(crate) fn rttbase_device_find(name: &str) -> Result<RtDevice, RTTError> {
    let name = CString::new(name);
    let ret;
    unsafe {
        ret = rt_device_find(name.str.as_ptr());
    }

    return if ret == 0 as *const CVoid {
        Err(RTTError::DeviceNotFound)
    } else {
        Ok(RtDevice(ret))
    };
}

pub(crate) fn rttbase_device_open(dev: *const CVoid, open_flag: u16) -> Result<(), RTTError> {
    unsafe {
        let ret = rt_device_open(dev, open_flag);

        return if ret != 0 {
            Err(RTTError::DeviceOpenFailed)
        } else {
            Ok(())
        };
    }
}

pub(crate) fn rttbase_device_close(dev: *const CVoid) -> Result<(), RTTError> {
    unsafe {
        let ret = rt_device_close(dev);

        return if ret != 0 {
            Err(RTTError::DeviceCloseFailed)
        } else {
            Ok(())
        };
    }
}

pub(crate) fn rttbase_device_write(
    dev: *const CVoid,
    pos: isize,
    buf: *const CVoid,
    size: usize,
) -> usize {
    unsafe { rt_device_write(dev, pos, buf, size) }
}

pub(crate) fn rttbase_device_read(
    dev: *const CVoid,
    pos: isize,
    buf: *mut CVoid,
    size: usize,
) -> usize {
    unsafe { rt_device_read(dev, pos, buf, size) }
}

pub(crate) fn rttbase_device_ctrl_config(
    dev: RTBaseDevice,
    arg: *const CVoid,
) -> Result<(), RTTError> {
    unsafe {
        let t = rt_device_control(dev, 3, arg);
        if t == 0 {
            Ok(())
        } else {
            Err(RTTError::DeviceConfigFailed)
        }
    }
}

pub(crate) fn rttbase_device_ctrl_suspend(
    dev: RTBaseDevice,
    arg: *const CVoid,
) -> Result<(), RTTError> {
    unsafe {
        if 0 == rt_device_control(dev, 2, arg) {
            Ok(())
        } else {
            Err(RTTError::DeviceConfigFailed)
        }
    }
}

pub(crate) fn rttbase_device_ctrl_resume(
    dev: RTBaseDevice,
    arg: *const CVoid,
) -> Result<(), RTTError> {
    unsafe {
        if 0 == rt_device_control(dev, 1, arg) {
            Ok(())
        } else {
            Err(RTTError::DeviceConfigFailed)
        }
    }
}

pub(crate) fn rttbase_device_ctrl_close(
    dev: RTBaseDevice,
    arg: *const CVoid,
) -> Result<(), RTTError> {
    unsafe {
        if 0 == rt_device_control(dev, 4, arg) {
            Ok(())
        } else {
            Err(RTTError::DeviceConfigFailed)
        }
    }
}

pub(crate) fn rttbase_device_ctrl(
    dev: RTBaseDevice,
    cmd: i32,
    arg: *const CVoid,
) -> Result<(), RTTError> {
    unsafe {
        if 0 == rt_device_control(dev, cmd, arg) {
            Ok(())
        } else {
            Err(RTTError::DeviceConfigFailed)
        }
    }
}

pub trait Open<Builder>: Sized {
    fn open(builder: &Builder) -> Result<Self, RTTError>;
}

pub trait Read<Out> {
    fn read(&self, size: usize) -> Out;
}
