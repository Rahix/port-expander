//! Support for the `CH422` "Remote 12-bit I/O expander for I2C-bus"

use core::marker::PhantomData;

use crate::mode::{Input, Output};

const WRITE_SET: u8 = 0x48 >> 1;
const WRITE_OUTPUT: u8 = 0x46 >> 1;
const WRITE_IO: u8 = 0x70 >> 1;
const READ_IO: u8 = 0x4D >> 1;

const FLAG_IO_ENABLE_OUTPUT: u8 = 1;
const FLAG_A_SCAN: u8 = 1 << 2;
const FLAG_OD_ENABLE: u8 = 1 << 4;
const FLAG_SLEEP: u8 = 1 << 7;

/// `CH422` "Remote 8-bit I/O expander for I2C-bus with interrupt"
pub struct Ch422<M>(M);

impl<I2C> Ch422<core::cell::RefCell<Driver<I2C, Input>>>
where
    I2C: crate::I2cBus,
{
    pub fn new(i2c: I2C) -> Self {
        Self::with_mutex(i2c)
    }
}

impl<I2C, M> Ch422<M>
where
    I2C: crate::I2cBus,
    M: crate::PortMutex<Port = Driver<I2C, Input>>,
{
    pub fn with_mutex(i2c: I2C) -> Self {
        Self(crate::PortMutex::create(Driver::new(i2c)))
    }

    pub fn enable_output<MOutput>(self) -> Result<Ch422<MOutput>, I2C::BusError>
    where
        MOutput: crate::PortMutex<Port = Driver<I2C, Output>>,
    {
        let driver = self.0.into_inner();
        let driver = driver.enable_output()?;
        Ok(Ch422(crate::PortMutex::create(driver)))
    }
}
impl<I2C, M> Ch422<M>
where
    I2C: crate::I2cBus,
    M: crate::PortMutex<Port = Driver<I2C, Output>>,
{
    pub fn disable_output<MInput>(self) -> Ch422<MInput>
    where
        MInput: crate::PortMutex<Port = Driver<I2C, Input>>,
    {
        let driver = self.0.into_inner();
        let driver = driver.disable_output();
        Ch422(crate::PortMutex::create(driver))
    }
}

impl<I2C, M, Mode> Ch422<M>
where
    I2C: crate::I2cBus,
    M: crate::PortMutex<Port = Driver<I2C, Mode>>,
{
    pub fn split(&mut self) -> Parts<'_, I2C, Mode, M> {
        Parts {
            io0: crate::Pin::new(0, &self.0),
            io1: crate::Pin::new(1, &self.0),
            io2: crate::Pin::new(2, &self.0),
            io3: crate::Pin::new(3, &self.0),
            io4: crate::Pin::new(4, &self.0),
            io5: crate::Pin::new(5, &self.0),
            io6: crate::Pin::new(6, &self.0),
            io7: crate::Pin::new(7, &self.0),
            o0: crate::Pin::new(8, &self.0),
            o1: crate::Pin::new(9, &self.0),
            o2: crate::Pin::new(10, &self.0),
            o3: crate::Pin::new(11, &self.0),
        }
    }
}

pub struct Parts<'a, I2C, Mode, M = core::cell::RefCell<Driver<I2C, Mode>>>
where
    I2C: crate::I2cBus,
    M: crate::PortMutex<Port = Driver<I2C, Mode>>,
{
    pub io0: crate::Pin<'a, Mode, M>,
    pub io1: crate::Pin<'a, Mode, M>,
    pub io2: crate::Pin<'a, Mode, M>,
    pub io3: crate::Pin<'a, Mode, M>,
    pub io4: crate::Pin<'a, Mode, M>,
    pub io5: crate::Pin<'a, Mode, M>,
    pub io6: crate::Pin<'a, Mode, M>,
    pub io7: crate::Pin<'a, Mode, M>,
    pub o0: crate::Pin<'a, crate::mode::Output, M>,
    pub o1: crate::Pin<'a, crate::mode::Output, M>,
    pub o2: crate::Pin<'a, crate::mode::Output, M>,
    pub o3: crate::Pin<'a, crate::mode::Output, M>,
}

pub struct Driver<I2C, Mode> {
    i2c: I2C,
    out: u8,
    io_mode: PhantomData<Mode>,
}

impl<I2C: crate::I2cBus> Driver<I2C, Input> {
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            out: 0xff,
            io_mode: PhantomData,
        }
    }
    pub fn enable_output(mut self) -> Result<Driver<I2C, Output>, I2C::BusError> {
        self.i2c.write(WRITE_SET, &[FLAG_IO_ENABLE_OUTPUT])?;
        Ok(Driver {
            i2c: self.i2c,
            out: self.out,
            io_mode: PhantomData,
        })
    }
}
impl<I2C> Driver<I2C, Output> {
    pub fn disable_output(self) -> Driver<I2C, Input> {
        Driver {
            i2c: self.i2c,
            out: self.out,
            io_mode: PhantomData,
        }
    }
}

impl<I2C: crate::I2cBus, Mode> crate::PortDriver for Driver<I2C, Mode> {
    type Error = I2C::BusError;

    fn set(&mut self, mask_high: u32, mask_low: u32) -> Result<(), Self::Error> {
        self.out |= mask_high as u8;
        self.out &= !mask_low as u8;
        self.i2c.write(WRITE_IO, &[self.out])?;
        Ok(())
    }

    fn is_set(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error> {
        Ok(((self.out as u32) & mask_high) | (!(self.out as u32) & mask_low))
    }

    fn get(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error> {
        let mut buf = [0x00];
        self.i2c.read(READ_IO, &mut buf)?;
        let in_ = buf[0] as u32;
        Ok((in_ & mask_high) | (!in_ & mask_low))
    }
}

#[cfg(test)]
mod tests {
    use core::cell::RefCell;

    use embedded_hal_mock::eh1::i2c as mock_i2c;

    use super::*;

    #[test]
    fn ch422() {
        let expectations = [
            mock_i2c::Transaction::write(WRITE_SET, vec![0b00000001]),
            mock_i2c::Transaction::write(WRITE_IO, vec![0b11111111]),
            mock_i2c::Transaction::write(WRITE_IO, vec![0b11111011]),
            mock_i2c::Transaction::read(READ_IO, vec![0b01000000]),
            mock_i2c::Transaction::read(READ_IO, vec![0b10111111]),
        ];
        let mut bus = mock_i2c::Mock::new(&expectations);

        let ch422 = super::Ch422::new(bus.clone());
        let mut ch422: Ch422<RefCell<_>> = ch422.enable_output().unwrap();
        let mut ch422_pins = ch422.split();

        ch422_pins.io2.set_high().unwrap();
        ch422_pins.io2.set_low().unwrap();

        let mut ch422: Ch422<RefCell<_>> = ch422.disable_output();

        let ch422_pins = ch422.split();

        assert!(ch422_pins.io6.is_high().unwrap());
        assert!(ch422_pins.io6.is_low().unwrap());

        bus.done();
    }
}
