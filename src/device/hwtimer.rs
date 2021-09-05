use crate::base::{CVoid, RTTError};
use crate::device::common::{
    rttbase_device_ctrl, rttbase_device_find, rttbase_device_open, RtDevice,
};
use crate::string::{String, ToString};

struct HwTimer {
    handle: RtDevice,
    mode: Mode,
    freq: Option<u32>,
}

#[derive(Copy, Clone)]
enum Mode {
    OneShot = 1,
    Period,
}

struct HwTimerBuilder {
    name: String,
    mode: Mode,
    freq: Option<u32>,
}

impl HwTimerBuilder {
    fn mode(&mut self, m: Mode) -> &mut Self {
        self.mode = m;
        self
    }

    fn freq(&mut self, f: u32) -> &mut Self {
        self.freq = Some(f);
        self
    }

    fn open(&self) -> Result<HwTimer, RTTError> {
        HwTimer::open(self)
    }
}

impl HwTimer {
    fn new(name: &str) -> HwTimerBuilder {
        HwTimerBuilder {
            name: name.to_string(),
            mode: Mode::OneShot,
            freq: None,
        }
    }

    fn open(builder: &HwTimerBuilder) -> Result<HwTimer, RTTError> {
        let dev;

        dev = rttbase_device_find(builder.name.as_str())?;
        rttbase_device_open(dev.raw(), 3)?;

        if let Some(f) = builder.freq {
            rttbase_device_ctrl(dev.raw(), 1, f as *const _)?;
        }

        rttbase_device_ctrl(dev.raw(), 4, builder.mode as u32 as *const _)?;

        Ok(HwTimer {
            handle: dev,
            mode: builder.mode,
            freq: builder.freq,
        })
    }

    fn set_freq(&self, f: u32) -> Result<(), RTTError> {
        rttbase_device_ctrl(self.handle.raw(), 1, f as *const _)
    }

    fn stop(&self) -> Result<(), RTTError> {
        rttbase_device_ctrl(self.handle.raw(), 2, 0 as *const CVoid)
    }

    fn set_mode(&self, m: Mode) -> Result<(), RTTError> {
        rttbase_device_ctrl(self.handle.raw(), 4, m as u32 as *const _)
    }

    fn write_and_start() {}

    fn read() {}

    fn set_rx_indicate() {}
}
