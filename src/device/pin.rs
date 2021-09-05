use crate::alloc::boxed::Box;
use crate::base::RTTError::DeviceWriteFailed;
use crate::base::{CVoid, RTBaseError, RTTError};
use crate::device::common::*;
use core::mem;

struct PIN {
    index: isize,
    mode: Mode,
}

struct IRQPin {
    pin: PIN,
    irq_func: Option<Box<Box<dyn FnMut()>>>,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Mode {
    Output,
    Input,
    InputPullUp,
    InputPullDown,
    OutputOD,
}

#[derive(Debug)]
enum IRQMode {
    Rising,
    Falling,
    RisingFalling,
    HighLevel,
    LowLevel,
}

#[derive(Debug)]
enum PinState {
    Low,
    High,
}

struct PinBuilder {
    index: isize,
    mode: Mode,
}

extern "C" {
    fn rt_pin_write(pin: isize, val: isize);
    fn rt_pin_read(pin: isize) -> i32;
    fn rt_pin_attach_irq(
        pin: i32,
        mode: u32,
        func: extern "C" fn(arg: *mut CVoid),
        arg: *mut CVoid,
    ) -> RTBaseError;
    fn rt_pin_detach_irq(pin: i32) -> RTBaseError;
    fn rt_pin_mode(pin: isize, mode: isize);
    fn rt_pin_irq_enable(pin: isize, mode: u32);
}

impl Open<PinBuilder> for PIN {
    fn open(builder: &PinBuilder) -> Result<Self, RTTError> {
        unsafe {
            rt_pin_mode(builder.index, builder.mode as isize);
        }
        Ok(PIN {
            index: builder.index,
            mode: builder.mode,
        })
    }
}

impl PIN {
    fn new(pin: isize) -> PinBuilder {
        PinBuilder {
            index: pin,
            mode: Mode::Output,
        }
    }

    fn pin_read(&self) -> Result<PinState, RTTError> {
        unsafe {
            Ok(if rt_pin_read(self.index) == 0 {
                PinState::Low
            } else {
                PinState::High
            })
        }
    }

    fn pin_write(&self, val: PinState) -> Result<(), RTTError> {
        return if self.mode == Mode::Input {
            Err(DeviceWriteFailed)
        } else {
            unsafe {
                rt_pin_write(self.index, val as isize);
            }
            Ok(())
        };
    }

    fn irq(self) -> Result<IRQPin, RTTError> {
        if self.mode == Mode::Output || self.mode == Mode::OutputOD {
            return Err(RTTError::DeviceOpenFailed);
        }
        Ok(IRQPin {
            pin: self,
            irq_func: None,
        })
    }
}

impl IRQPin {
    fn attach_irq<T>(&mut self, func: T, mode: IRQMode) -> &mut IRQPin
    where
        T: FnMut() + 'static,
    {
        Self::_attach_irq(self, Box::new(func), mode);
        self
    }

    fn enable(&self) {
        unsafe {
            rt_pin_irq_enable(self.pin.index, 1);
        }
    }

    fn disable(&self) {
        unsafe {
            rt_pin_irq_enable(self.pin.index, 0);
        }
    }

    fn _attach_irq(&mut self, func: Box<dyn FnMut()>, mode: IRQMode) {
        let p = Box::new(func);
        let param = &*p as *const _ as *mut _;

        extern "C" fn f(arg: *mut CVoid) {
            unsafe {
                let mut run = Box::from_raw(arg as *mut Box<dyn FnMut()>);
                run();
                mem::forget(run);
            }
        }
        unsafe {
            if 0 != rt_pin_attach_irq(self.pin.index as i32, mode as u32, f, param) {
                self.irq_func = None;
                return;
            }
        }
        self.irq_func = Some(p);
    }
}

impl Drop for IRQPin {
    fn drop(&mut self) {
        if let Some(_) = self.irq_func {
            unsafe {
                rt_pin_detach_irq(self.pin.index as i32);
            }
        }
    }
}

impl PinBuilder {
    fn mode(&mut self, m: Mode) -> &mut Self {
        self.mode = m;
        self
    }

    fn open(&self) -> Result<PIN, RTTError> {
        PIN::open(&self)
    }
}
