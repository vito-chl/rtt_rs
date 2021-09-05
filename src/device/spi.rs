//! # Example
//! ```
//! use crate::rtt_rs::device::spi;
//! use rtt_rs::Deref;
//!
//! /* 加入少量消息 */
//! let mut a = Vec::<u8>::new();
//! /* 放入消息中开始传输时，一定要resize好大小 */
//! a.resize(2, 0);
//! let mut b = a.clone();
//! let mut c = a.clone();
//! /* 将发送三条请求，每次接受两个字节，分别存入 Vec: a b c 中 */
//! let msg = spi::SpiMsg::new_rec(&mut a).append_rec(&mut b).append_rec(&mut c);
//!
//! /* 批量加入消息 */
//! let mut msg_vec = Vec::new();
//! for _ in 0..5 {
//!     msg_vec.push(a.clone())
//! }
//! let mut head_msg = a.clone();
//! let mut head = spi::SpiMsg::new_rec(&mut head_msg);
//! let mut head_ptr = &mut head;
//! for mut i in msg_vec {
//!     head_ptr = head_ptr.append_rec(&mut i);
//! }
//!
//! /* 仅发送一个字节 */
//! let mut raw_msg = Vec::new();
//! raw_msg.push('a' as u8);
//!
//! let t_msg = spi::SpiMsg::new_send(&raw_msg);
//! let dev = spi::SPI::new_open("xxx").unwrap();
//! dev.transfer(t_msg).unwrap();
//!
//! /* 一条请求发送多个字节 */
//! let mut raw_msg_bytes = Vec::new();
//! raw_msg_bytes.copy_from_slice("abc".as_bytes().slice());
//! let t_msg = spi::SpiMsg::new_send(&raw_msg_bytes);
//! dev.transfer(t_msg).unwrap();
//! ```

use crate::base::*;
use crate::device::*;
use crate::vec;
use crate::vec::*;
use crate::{min, Box};

extern "C" {
    fn rt_spi_transfer_message(dev: *const CVoid, msg: *mut CVoid) -> usize;
}

pub struct SPI {
    handle: common::RtDevice,
}

impl SPI {
    pub fn new_open(name: &str) -> Result<SPI, RTTError> {
        let dev;
        dev = common::rttbase_device_find(name)?;
        Ok(SPI { handle: dev })
    }

    /// This function can send/receive a list messages
    /// In each message, the length of send-msg should be equal with rec-msg's
    /// if those is different, the smaller will be used
    pub fn transfer(&self, msg: SpiMsg) -> Result<(), RTTError> {
        let mut inner_msg_vec = Vec::new();
        let mut length_temp = 4096;

        let mut msg_ptr = Some(Box::new(msg));
        while let Some(msg_for_trans) = msg_ptr {
            let inner_msg = Box::new(InnerSpiMsg {
                send_buf: match msg_for_trans.send.as_ref() {
                    Some(msg) => {
                        length_temp = msg.len();
                        msg.as_ptr() as *const CVoid
                    }
                    None => 0 as *const CVoid,
                },
                rec_buf: match msg_for_trans.rec.as_ref() {
                    Some(msg) => {
                        length_temp = min(length_temp, msg.len());
                        msg.as_ptr() as *mut CVoid
                    }
                    None => 0 as *mut CVoid,
                },
                length: length_temp,
                next: 0 as _,
                flag: 0,
            });
            inner_msg_vec.push(inner_msg);
            msg_ptr = msg_for_trans.next
        }

        // link msg
        for i in 1..inner_msg_vec.len() {
            inner_msg_vec[i - 1].as_mut().next =
                inner_msg_vec[i].as_ref() as *const _ as *mut InnerSpiMsg;
        }

        // cs_take: 1bits
        // cs_release: 1bits
        // begin: 01 take:1 release:0
        // end: 10 take:0 release:1
        inner_msg_vec[0].as_mut().flag = 1;
        let end = inner_msg_vec.len() - 1;
        inner_msg_vec[end].as_mut().flag |= 3;

        unsafe {
            if 0 != rt_spi_transfer_message(
                self.handle.raw(),
                inner_msg_vec[0].as_mut() as *mut _ as *mut _,
            ) {
                Err(RTTError::DeviceTransFailed)
            } else {
                Ok(())
            }
        }
    }

    pub fn send_bytes(&self, buf: Vec<u8>) -> Result<(), RTTError> {
        let msg = SpiMsg::new_send(&buf);
        self.transfer(msg)
    }

    pub fn send_byte(&self, b: u8) -> Result<(), RTTError> {
        let mut s_buf = SpiMsg::make_vec();
        s_buf.clear();
        s_buf.push(b);

        let msg = SpiMsg::new_send(&s_buf);
        self.transfer(msg)
    }

    pub fn receive_byte(&self) -> Result<u8, RTTError> {
        let msg = self.receive_bytes(1)?;
        Ok(msg[0])
    }

    pub fn receive_bytes(&self, len: usize) -> Result<Vec<u8>, RTTError> {
        let mut rec = Vec::new();
        rec.resize(len, 0 as u8);

        let msg = SpiMsg::new_rec(&mut rec);
        self.transfer(msg)?;

        Ok(rec)
    }

    pub fn trans_byte(&self, b: u8) -> Result<u8, RTTError> {
        let mut r_buf = vec![0 as u8];
        let s_buf = vec![b];

        let msg = SpiMsg::new(&s_buf, &mut r_buf);

        self.transfer(msg)?;
        Ok(r_buf[0])
    }
}

pub struct SpiMsg<'a> {
    send: Option<&'a Vec<u8>>,
    rec: Option<&'a mut Vec<u8>>,
    next: Option<Box<SpiMsg<'a>>>,
}

impl<'a> SpiMsg<'a> {
    /// Make a vec which length is 8
    pub fn make_vec() -> Vec<u8> {
        let mut ret = Vec::new();
        ret.resize(8, 0 as u8);
        ret
    }

    pub fn make_vec_pair() -> (Vec<u8>, Vec<u8>) {
        let mut ret = Vec::new();
        ret.resize(8, 0 as u8);
        (ret.clone(), ret)
    }

    fn inner_new(s: Option<&'a Vec<u8>>, r: Option<&'a mut Vec<u8>>) -> SpiMsg<'a> {
        SpiMsg {
            send: s,
            rec: r,
            next: None,
        }
    }

    /// New a message with send-buf and receive-buf
    pub fn new(s: &'a Vec<u8>, r: &'a mut Vec<u8>) -> SpiMsg<'a> {
        if s.len() > r.len() {
            r.resize(s.len(), 0);
        }
        Self::inner_new(Some(s), Some(r))
    }

    pub fn new_send(s: &'a Vec<u8>) -> SpiMsg {
        Self::inner_new(Some(s), None)
    }

    pub fn new_rec(r: &'a mut Vec<u8>) -> SpiMsg {
        Self::inner_new(None, Some(r))
    }

    fn inner_append(&mut self, s: Option<&'a Vec<u8>>, r: Option<&'a mut Vec<u8>>) -> &mut Self {
        let n = Box::new(SpiMsg {
            send: s,
            rec: r,
            next: None,
        });

        self.next = Some(n);
        self.next.as_mut().unwrap().as_mut()
    }

    pub fn append(&mut self, s: &'a Vec<u8>, r: &'a mut Vec<u8>) -> &mut Self {
        self.inner_append(Some(s), Some(r))
    }

    pub fn append_send(&mut self, s: &'a Vec<u8>) -> &mut Self {
        self.inner_append(Some(s), None)
    }

    pub fn append_rec(&mut self, r: &'a mut Vec<u8>) -> &mut Self {
        self.inner_append(None, Some(r))
    }
}

#[repr(C)]
struct InnerSpiMsg {
    send_buf: *const CVoid,
    rec_buf: *mut CVoid,
    length: usize,
    next: *mut InnerSpiMsg,
    flag: u32,
}
