//! Controller Area Network, CAN

use crate::base::{CVoid, RTTError};
use crate::device::common::*;
use crate::string::{String, ToString};
use crate::vec::*;

use crate::mem;
use bitfield::*;

use super::common::rttbase_device_close;
use super::common::rttbase_device_find;
use super::common::rttbase_device_open;
use super::common::rttbase_device_read;
use super::common::rttbase_device_write;
use crate::device::can::CanBaud::CAN1MBaud;
use crate::device::can::CanMode::Normal;

#[derive(Copy, Clone, Debug)]
enum CanBaud {
    CAN1MBaud = 0, // 1 MBit/sec
    CAN800kBaud,   // 800 kBit/sec
    CAN500kBaud,   // 500 kBit/sec
    CAN250kBaud,   // 250 kBit/sec
    CAN125kBaud,   // 125 kBit/sec
    CAN100kBaud,   // 100 kBit/sec
    CAN50kBaud,    // 50 kBit/sec
    CAN20kBaud,    // 20 kBit/sec
    CAN10kBaud,    // 10 kBit/sec
}

#[derive(Copy, Clone, Debug)]
enum CanMode {
    Normal = 0,
    Listen,
    LoopBack,
    LoopBackListen,
}

pub struct CAN {
    handle: RtDevice,
}

impl Drop for CAN {
    fn drop(&mut self) {
        rttbase_device_close(self.handle.0).unwrap();
    }
}

impl CAN {
    fn new(name: &str) -> CANBuilder {
        CANBuilder {
            name: name.to_string(),
            baud_rate: CAN1MBaud,
            mode: Normal,
            int_rx: true,
            int_tx: false,
        }
    }

    fn write() {
        /* TODO: Send data by can device */
    }
}

impl Open<CANBuilder> for CAN {
    fn open(builder: &CANBuilder) -> Result<CAN, RTTError> {
        let t = rttbase_device_find(builder.name.as_str())?;

        let mut open_flag: u16 = 0;

        open_flag |= if builder.int_tx {
            0x400 /* INT mode on Tx */
        } else {
            0
        };
        open_flag |= if builder.int_rx {
            0x100 /* INT mode on Rx */
        } else {
            0x100 /* INT mode on Rx */
        };

        rttbase_device_open(t.0, open_flag)?;

        let ret = CAN { handle: t };

        rttbase_device_ctrl(t.0, 0x14, builder.baud_rate as u32 as *const CVoid)?;

        rttbase_device_ctrl(t.0, 0x15, builder.mode as u32 as *const CVoid)?;

        Ok(ret)
    }
}

impl Read<(u32, Vec<u8>)> for CAN {
    fn read(&self, _size: usize) -> (u32, Vec<u8>) {
        let mut msg = InnerMsg {
            flag1: 0,
            flag2: 0,
            data: [0; 8],
        };
        let len = rttbase_device_read(
            self.handle.raw() as *const CVoid,
            0,
            &mut msg as *mut _ as *mut _,
            mem::size_of::<InnerMsg>(),
        );
        let mut ret: Vec<u8> = Vec::new();
        ret.resize(len, 0);

        for i in 0..len {
            ret.push(msg.data[i])
        }
        (len as u32, ret)
    }
}

pub struct CANBuilder {
    name: String,
    baud_rate: CanBaud,
    mode: CanMode,
    int_rx: bool,
    int_tx: bool,
}

impl CANBuilder {
    fn baud_rate(&mut self, b: CanBaud) -> &mut Self {
        self.baud_rate = b;
        self
    }

    fn mode(&mut self, m: CanMode) -> &mut Self {
        self.mode = m;
        self
    }

    fn open(&self) -> Result<CAN, RTTError> {
        CAN::open(self)
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct InnerMsg {
    flag1: u32,
    flag2: u32,
    data: [u8; 8],
}

trait Write {
    fn write_to_can(&self, _bus: &CAN) -> Result<(), RTTError> {
        Err(RTTError::FuncUnDefine)
    }
}

pub struct CanMsgData {
    pub id: u32,
    pub expansion: bool,
    pub data: Vec<u8>,
}

#[derive(Copy, Clone)]
pub struct CanMsgRemote {
    pub id: u32,
    pub expansion: bool,
}

impl Write for CanMsgData {
    fn write_to_can(&self, _bus: &CAN) -> Result<(), RTTError> {
        let mut msg = InnerMsg {
            flag1: 0,
            flag2: 0,
            data: [0; 8],
        };

        bitfield! {
            pub struct FlagA(u32);
            _, set_id: 28, 0;
            _, set_ide: 29;
            _, set_rtr: 30;
        }

        let mut f1 = FlagA(0);
        f1.set_id(self.id);
        f1.set_ide(self.expansion);

        bitfield! {
            pub struct FlagB(u32);
            _, set_len: 7, 0;
            _, set_priv: 15, 8;
            _, set_hdr: 23, 16;
        }

        let mut f2 = FlagB(0);
        f2.set_len(self.data.len() as u32);

        msg.flag1 = f1.0;
        msg.flag2 = f2.0;

        rttbase_device_write(_bus.handle.raw(), 0, &mut msg as *mut _ as *const CVoid, 0);
        Ok(())
    }
}

impl Write for CanMsgRemote {
    fn write_to_can(&self, _bus: &CAN) -> Result<(), RTTError> {
        let mut msg = InnerMsg {
            flag1: 0,
            flag2: 0,
            data: [0; 8],
        };

        bitfield! {
            pub struct FlagA(u32);
            _, set_id: 28, 0;
            _, set_ide: 29;
            _, set_rtr: 30;
        }

        let mut f1 = FlagA(0);
        f1.set_id(self.id);
        f1.set_ide(self.expansion);
        f1.set_rtr(true);

        msg.flag1 = f1.0;

        rttbase_device_write(_bus.handle.raw(), 0, &mut msg as *mut _ as *const CVoid, 0);
        Ok(())
    }
}
