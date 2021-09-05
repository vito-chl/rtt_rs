//! UART, more imfomation in `struct UART`
//!
//! # Examples:
//! ```
//! use rtt_rs::device::uart::UART;
//! use rtt_rs::device::uart::BaudRate::BaudRate115200;
//!
//! let dev = UART::new("uart1")
//!     .baud_rate(BaudRate115200)
//!     .rx_dma(true).open().unwrap();
//!
//! dev.write_bytes(0 , "abcdefg".as_bytes());
//!
//! /* read 10 bytes */
//! let a = dev.read_bytes(0, 10);
//! ```

use crate::base::{CVoid, RTTError};
use crate::device::common::*;
use crate::string::String;
use crate::vec::*;
use bitfield::*;
use core::mem;

pub struct UART {
    handle: RtDevice,
}

#[derive(Copy, Clone)]
pub enum BaudRate {
    BaudRate2400 = 2400,
    BaudRate4800 = 4800,
    BaudRate9600 = 9600,

    BaudRate19200 = 19200,
    BaudRate38400 = 38400,
    BaudRate57600 = 57600,

    BaudRate115200 = 115200,
    BaudRate230400 = 230400,
    BaudRate460800 = 460800,
    BaudRate921600 = 921600,

    BaudRate2000000 = 2000000,
    BaudRate3000000 = 3000000,
}
#[derive(Copy, Clone)]
pub enum DataBits {
    DataBits5 = 5,
    DataBits6 = 6,
    DataBits7 = 7,
    DataBits8 = 8,
    DataBits9 = 9,
}
#[derive(Copy, Clone)]
pub enum StopBits {
    StopBits1 = 0,
    StopBits2 = 1,
    StopBits3 = 2,
    StopBits4 = 3,
}
#[derive(Copy, Clone)]
pub enum Parity {
    None = 0,
    ODD = 1,
    EVEN = 2,
}
#[derive(Copy, Clone)]
pub enum BitOrder {
    LSB = 0,
    MSB = 1,
}
#[derive(Copy, Clone)]
pub enum Invert {
    Normal = 0,
    Inverted = 1,
}

pub struct UARTBuilder {
    name: String,

    baud_rate: BaudRate,
    data_bits: DataBits,
    stop_bits: StopBits,
    parity: Parity,
    bit_order: BitOrder,
    invert: Invert,
    buf_size: u16,

    rx_dma: bool,
    tx_dma: bool,
}

struct UARTBuilderInner {
    baud_rate: u32,
    flag: u32,
}

impl UARTBuilder {
    pub fn baud_rate(&mut self, val: BaudRate) -> &mut UARTBuilder {
        self.baud_rate = val;
        self
    }
    pub fn data_bits(&mut self, val: DataBits) -> &mut UARTBuilder {
        self.data_bits = val;
        self
    }
    pub fn stop_bits(&mut self, val: StopBits) -> &mut UARTBuilder {
        self.stop_bits = val;
        self
    }
    pub fn parity(&mut self, val: Parity) -> &mut UARTBuilder {
        self.parity = val;
        self
    }
    pub fn bit_order(&mut self, val: BitOrder) -> &mut UARTBuilder {
        self.bit_order = val;
        self
    }
    pub fn invert(&mut self, val: Invert) -> &mut UARTBuilder {
        self.invert = val;
        self
    }
    pub fn tx_dma(&mut self, f: bool) -> &mut UARTBuilder {
        self.tx_dma = f;
        self
    }
    pub fn rx_dma(&mut self, f: bool) -> &mut UARTBuilder {
        self.rx_dma = f;
        self
    }

    pub fn open(&self) -> Result<UART, RTTError> {
        UART::open(&self)
    }
}

impl Open<UARTBuilder> for UART {
    fn open(builder: &UARTBuilder) -> Result<UART, RTTError> {
        let t = rttbase_device_find(builder.name.as_str())?;

        let mut open_flag: u16 = 0;

        open_flag |= if builder.tx_dma {
            0x800 /* DMA mode on Tx */
        } else {
            0
        };
        open_flag |= if builder.rx_dma {
            0x200 /* DMA mode on Rx */
        } else {
            0x100 /* INT mode on Rx */
        };
        rttbase_device_open(t.0, open_flag)?;

        let ret = UART { handle: t };
        ret.config(builder)?;
        Ok(ret)
    }
}

impl UART {
    /// New a uart device
    /// # Note:
    /// If you want to change the configuration of the UART.
    ///
    /// Please close the current UART and reconfigure it.
    ///
    /// # Examples:
    /// ```
    /// use rtt_rs::device::uart::UART;
    /// use rtt_rs::device::uart::BaudRate::BaudRate115200;
    ///
    /// let dev = UART::new("uart1")
    ///     .baud_rate(BaudRate115200)
    ///     .rx_dma(true).open().unwrap();
    ///
    /// dev.write_bytes(0 , "abcdefg".as_bytes());
    ///
    /// /* read 10 bytes */
    /// let a = dev.read_bytes(0, 10);
    /// ```
    pub fn new(name: &str) -> UARTBuilder {
        UARTBuilder {
            name: String::from(name),
            baud_rate: BaudRate::BaudRate115200,
            data_bits: DataBits::DataBits8,
            stop_bits: StopBits::StopBits1,
            parity: Parity::None,
            bit_order: BitOrder::LSB,
            invert: Invert::Normal,
            buf_size: 64,
            /* not use dma default */
            tx_dma: false,
            rx_dma: false,
        }
    }

    fn config(&self, cfg: &UARTBuilder) -> Result<(), RTTError> {
        let mut inner_cfg = UARTBuilderInner {
            baud_rate: cfg.baud_rate as u32,
            flag: 0,
        };

        bitfield! {
            pub struct Flag(u32);
            _, data_bits: 3, 0;
            _, stop_bits: 5, 4;
            _, parity: 7, 6;
            _, bitorder: 8;
            _, invert: 9;
            _, bufsz: 25, 10;
        }

        let mut set_flag = Flag(0);
        set_flag.data_bits(cfg.data_bits as u32);
        set_flag.stop_bits(cfg.stop_bits as u32);
        set_flag.parity(cfg.parity as u32);
        set_flag.bitorder(cfg.bit_order as u32 != 0);
        set_flag.invert(cfg.invert as u32 != 0);
        set_flag.bufsz(cfg.buf_size as u32);

        inner_cfg.flag = set_flag.0;

        rttbase_device_ctrl_config(self.handle.0, &mut inner_cfg as *mut _ as *const _)?;

        Ok(())
    }

    /// Length information is included in Vec
    pub fn read_bytes(&self, pos: isize, size: usize) -> Result<Vec<u8>, RTTError> {
        let mut buf = Vec::<u8>::new();
        buf.resize(size, 0);
        let len = rttbase_device_read(self.handle.0, pos, buf.as_mut_ptr() as *mut CVoid, size);
        if len > size {
            Err(RTTError::DeviceReadFailed)
        } else {
            unsafe {
                buf.set_len(len);
            }
            Ok(buf)
        }
    }

    /// You can receive an object by this function
    ///
    /// # Note:
    /// this function will not deal with big/little endian problem
    pub fn read_obj<T>(&self, pos: isize, obj: &mut T) -> Result<(), RTTError> {
        let size = mem::size_of::<T>();

        return if size != rttbase_device_read(self.handle.0, pos, obj as *mut T as *mut CVoid, size)
        {
            Err(RTTError::DeviceReadFailed)
        } else {
            Ok(())
        };
    }

    /// Length information is included in Vec
    pub fn write_bytes(&self, pos: isize, buf: &[u8]) -> usize {
        rttbase_device_write(self.handle.0, pos, buf.as_ptr() as *const CVoid, buf.len())
    }

    /// You can send an object by this function
    ///
    /// # Note:
    /// this function will not deal with big/little endian problem
    pub fn write_obj<T>(&self, pos: isize, obj: &T) -> Result<(), RTTError> {
        let size = mem::size_of::<T>();

        return if size
            != rttbase_device_write(
                self.handle.raw(),
                pos,
                obj as *const T as *const CVoid,
                size,
            ) {
            Err(RTTError::DeviceWriteFailed)
        } else {
            Ok(())
        };
    }

    /// get handle in rtthread system
    ///
    /// this handle is same as `dev` in call back function
    pub fn raw_handle(&self) -> *const CVoid {
        self.handle.0
    }

    /// this methrod while register func to system
    ///
    /// The registered function will be called in the interrupt service function
    ///
    /// # Unimportant
    /// I originally hoped to use closures to achieve this function,
    /// But the API in RT thread does not support me to pass closures as parameters.
    /// so I can only use basic functions as the callback function.
    /// Although we can use a global HashMap to achieve similar functions,
    /// But this will lose the performance of the system.
    ///
    /// # Note:
    /// It should be noted that this function can only do some simple operations,
    /// such as sending semaphores.
    /// because this function is called in the interrupt function at RT thread.
    /// When using it, you should pay attention to whether there will be deadlock.
    /// For example, if you execute printing operation, there may be deadlock.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use rtt_rs::base::CVoid;
    /// extern "C" fn cb_func(dev:*const CVoid, buf:*const CVoid) -> isize
    /// {
    ///     /* release sem */
    ///     unimplemented!()
    /// }
    /// ```
    pub fn set_rx_indicate(
        &self,
        func: extern "C" fn(dev: *const CVoid, size: usize) -> isize,
    ) -> Result<(), RTTError> {
        unsafe {
            if 0 == rt_device_set_rx_indicate(self.handle.0, func) {
                Ok(())
            } else {
                Err(RTTError::DeviceSetRxCallBackFailed)
            }
        }
    }

    /// this methrod while register func to system
    ///
    /// The registered function will be called in the interrupt service function
    /// # Note:
    /// Only can be use in mode `DMA tx`
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use rtt_rs::base::CVoid;
    /// extern "C" fn cb_func(dev:*const CVoid, buf:*const CVoid) -> isize
    /// {
    ///     /* release sem */
    ///     unimplemented!()
    /// }
    /// ```
    pub fn set_tx_complete(
        &self,
        func: extern "C" fn(dev: *const CVoid, buf: *const CVoid) -> isize,
    ) -> Result<(), RTTError> {
        unsafe {
            if 0 == rt_device_set_tx_complete(self.handle.0, func) {
                Ok(())
            } else {
                Err(RTTError::DeviceSetTxCallBackFailed)
            }
        }
    }
}

impl Read<Vec<u8>> for UART {
    fn read(&self, size: usize) -> Vec<u8> {
        Self::read_bytes(self, 0, size).unwrap()
    }
}

impl Drop for UART {
    fn drop(&mut self) {
        rttbase_device_close(self.handle.0).unwrap();
    }
}
