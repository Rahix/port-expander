//! Support for the `PCF8575` "Remote 16-bit I/O expander for I2C-bus with interrupt"

/// `PCF8575` "Remote 16-bit I/O expander for I2C-bus with interrupt"
pub struct Pcf8575<M>(M);

impl<I2C> Pcf8575<shared_bus::NullMutex<Driver<I2C>>>
where
    I2C: crate::I2cBus,
{
    pub fn new(i2c: I2C, a0: bool, a1: bool, a2: bool) -> Self {
        Self::with_mutex(i2c, a0, a1, a2)
    }
}

impl<I2C, M> Pcf8575<M>
where
    I2C: crate::I2cBus,
    M: shared_bus::BusMutex<Bus = Driver<I2C>>,
{
    pub fn with_mutex(i2c: I2C, a0: bool, a1: bool, a2: bool) -> Self {
        Self(shared_bus::BusMutex::create(Driver::new(i2c, a0, a1, a2)))
    }

    pub fn split(&mut self) -> Parts<'_, I2C, M> {
        Parts {
            p00: crate::Pin::new(0, &self.0),
            p01: crate::Pin::new(1, &self.0),
            p02: crate::Pin::new(2, &self.0),
            p03: crate::Pin::new(3, &self.0),
            p04: crate::Pin::new(4, &self.0),
            p05: crate::Pin::new(5, &self.0),
            p06: crate::Pin::new(6, &self.0),
            p07: crate::Pin::new(7, &self.0),
            p10: crate::Pin::new(8, &self.0),
            p11: crate::Pin::new(9, &self.0),
            p12: crate::Pin::new(10, &self.0),
            p13: crate::Pin::new(11, &self.0),
            p14: crate::Pin::new(12, &self.0),
            p15: crate::Pin::new(13, &self.0),
            p16: crate::Pin::new(14, &self.0),
            p17: crate::Pin::new(15, &self.0),
        }
    }
}

pub struct Parts<'a, I2C, M = shared_bus::NullMutex<Driver<I2C>>>
where
    I2C: crate::I2cBus,
    M: shared_bus::BusMutex<Bus = Driver<I2C>>,
{
    pub p00: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub p01: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub p02: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub p03: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub p04: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub p05: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub p06: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub p07: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub p10: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub p11: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub p12: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub p13: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub p14: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub p15: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub p16: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub p17: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
}

pub struct Driver<I2C> {
    i2c: I2C,
    out: [u8; 2],
    addr: u8,
}

impl<I2C> Driver<I2C> {
    pub fn new(i2c: I2C, a0: bool, a1: bool, a2: bool) -> Self {
        Self {
            i2c,
            out: [0xff; 2],
            addr: 0x20 | ((a2 as u8) << 2) | ((a1 as u8) << 1) | (a0 as u8),
        }
    }
}

impl<I2C: crate::I2cBus> crate::PortDriver for Driver<I2C> {
    type Error = I2C::BusError;

    fn set(&mut self, mask_high: u32, mask_low: u32) -> Result<(), Self::Error> {
        let mut out = u16::from_le_bytes(self.out);
        out |= mask_high as u16;
        out &= !mask_low as u16;

        self.out = out.to_le_bytes();

        self.i2c.write(self.addr, &self.out)?;
        Ok(())
    }

    fn is_set(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error> {
        let out = u16::from_le_bytes(self.out);

        Ok(((out as u32) & mask_high) | (!(out as u32) & mask_low))
    }

    fn get(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error> {
        let mut buf = [0x00; 2];
        self.i2c.read(self.addr, &mut buf)?;
        let in_ = u16::from_le_bytes(buf) as u32;

        Ok((in_ & mask_high) | (!in_ & mask_low))
    }
}

#[cfg(test)]
mod tests {
    use embedded_hal_mock::eh1::i2c as mock_i2c;

    #[test]
    fn pcf8575() {
        let expectations = [
            mock_i2c::Transaction::write(0x21, vec![0b11111111, 0b11111111]),
            mock_i2c::Transaction::write(0x21, vec![0b11111011, 0b11111111]),
            mock_i2c::Transaction::read(0x21, vec![0b01000000, 0b00000000]),
            mock_i2c::Transaction::read(0x21, vec![0b10111111, 0b11111111]),
            mock_i2c::Transaction::write(0x21, vec![0b11111011, 0b11111111]),
            mock_i2c::Transaction::write(0x21, vec![0b11111011, 0b11111011]),
            mock_i2c::Transaction::read(0x21, vec![0b00000000, 0b01000000]),
            mock_i2c::Transaction::read(0x21, vec![0b11111111, 0b10111111]),
        ];
        let mut bus = mock_i2c::Mock::new(&expectations);

        let mut pcf = super::Pcf8575::new(bus.clone(), true, false, false);
        let mut pcf_pins = pcf.split();

        pcf_pins.p02.set_high().unwrap();
        pcf_pins.p02.set_low().unwrap();

        assert!(pcf_pins.p06.is_high().unwrap());
        assert!(pcf_pins.p06.is_low().unwrap());

        pcf_pins.p12.set_high().unwrap();
        pcf_pins.p12.set_low().unwrap();

        assert!(pcf_pins.p16.is_high().unwrap());
        assert!(pcf_pins.p16.is_low().unwrap());

        bus.done();
    }
}
