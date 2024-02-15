//! Support for the Maxim 7321 I2C 8-Port Open Drain port expander
pub struct Max7321<M>(M);

/// MAX7321 "I2C Port Expander with 8 Open-Drain I/Os"
impl<I2C> Max7321<core::cell::RefCell<Driver<I2C>>>
where
    I2C: crate::I2cBus,
{
    pub fn new(i2c: I2C, a3: bool, a2: bool, a1: bool, a0: bool) -> Self {
        Self::with_mutex(i2c, a3, a2, a1, a0)
    }
}

impl<I2C, M> Max7321<M>
where
    I2C: crate::I2cBus,
    M: crate::PortMutex<Port = Driver<I2C>>,
{
    pub fn with_mutex(i2c: I2C, a3: bool, a2: bool, a1: bool, a0: bool) -> Self {
        Self(crate::PortMutex::create(Driver::new(i2c, a3, a2, a1, a0)))
    }

    pub fn split(&mut self) -> Parts<'_, I2C, M> {
        Parts {
            p0: crate::Pin::new(0, &self.0),
            p1: crate::Pin::new(1, &self.0),
            p2: crate::Pin::new(2, &self.0),
            p3: crate::Pin::new(3, &self.0),
            p4: crate::Pin::new(4, &self.0),
            p5: crate::Pin::new(5, &self.0),
            p6: crate::Pin::new(6, &self.0),
            p7: crate::Pin::new(7, &self.0),
        }
    }
}

pub struct Parts<'a, I2C, M = core::cell::RefCell<Driver<I2C>>>
where
    I2C: crate::I2cBus,
    M: crate::PortMutex<Port = Driver<I2C>>,
{
    pub p0: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub p1: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub p2: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub p3: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub p4: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub p5: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub p6: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub p7: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
}

pub struct Driver<I2C> {
    i2c: I2C,
    out: u8,
    addr: u8,
}

impl<I2C> Driver<I2C> {
    pub fn new(i2c: I2C, a3: bool, a2: bool, a1: bool, a0: bool) -> Self {
        let addr = 0x60 | ((a3 as u8) << 3) | ((a2 as u8) << 2) | ((a1 as u8) << 1) | (a0 as u8);
        Self {
            i2c,
            out: 0xff,
            addr,
        }
    }
}

impl<I2C: crate::I2cBus> crate::PortDriver for Driver<I2C> {
    type Error = I2C::BusError;

    fn set(&mut self, mask_high: u32, mask_low: u32) -> Result<(), Self::Error> {
        self.out |= mask_high as u8;
        self.out &= !mask_low as u8;
        self.i2c.write(self.addr, &[self.out])?;
        Ok(())
    }

    fn is_set(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error> {
        Ok(((self.out as u32) & mask_high) | (!(self.out as u32) & mask_low))
    }

    fn get(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error> {
        let mut buf = [0x00];
        self.i2c.read(self.addr, &mut buf)?;
        let in_ = buf[0] as u32;
        Ok((in_ & mask_high) | (!in_ & mask_low))
    }
}

#[cfg(test)]
mod tests {
    use embedded_hal_mock::eh1::i2c as mock_i2c;

    #[test]
    fn max7321() {
        let expectations = [
            mock_i2c::Transaction::write(0b01101101, vec![0b11111111]),
            mock_i2c::Transaction::write(0b01101101, vec![0b11111011]),
            mock_i2c::Transaction::read(0b01101101, vec![0b01000000]),
            mock_i2c::Transaction::read(0b01101101, vec![0b10111111]),
        ];
        let mut bus = mock_i2c::Mock::new(&expectations);

        let mut max = super::Max7321::new(bus.clone(), true, true, false, true);
        let mut max_pins = max.split();

        max_pins.p2.set_high().unwrap();
        max_pins.p2.set_low().unwrap();

        assert!(max_pins.p6.is_high().unwrap());
        assert!(max_pins.p6.is_low().unwrap());

        bus.done();
    }
}
