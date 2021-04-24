//! Support for the `PCF8574` & `PCF8574A` "Remote 8-bit I/O expander for I2C-bus with interrupt"

/// `PCF8574` "Remote 8-bit I/O expander for I2C-bus with interrupt"
pub struct Pcf8574<M>(M);
/// `PCF8574A` "Remote 8-bit I/O expander for I2C-bus with interrupt"
pub struct Pcf8574a<M>(M);

impl<I2C> Pcf8574<shared_bus::NullMutex<Driver<I2C>>>
where
    I2C: crate::I2cBus,
{
    pub fn new(i2c: I2C, a0: bool, a1: bool, a2: bool) -> Self {
        Self::with_mutex(i2c, a0, a1, a2)
    }
}

impl<I2C> Pcf8574a<shared_bus::NullMutex<Driver<I2C>>>
where
    I2C: crate::I2cBus,
{
    pub fn new(i2c: I2C, a0: bool, a1: bool, a2: bool) -> Self {
        Self::with_mutex(i2c, a0, a1, a2)
    }
}

impl<I2C, M> Pcf8574<M>
where
    I2C: crate::I2cBus,
    M: shared_bus::BusMutex<Bus = Driver<I2C>>,
{
    pub fn with_mutex(i2c: I2C, a0: bool, a1: bool, a2: bool) -> Self {
        Self(shared_bus::BusMutex::create(Driver::new(
            i2c, false, a0, a1, a2,
        )))
    }

    pub fn split<'a>(&'a mut self) -> Parts<'a, I2C, M> {
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

impl<I2C, M> Pcf8574a<M>
where
    I2C: crate::I2cBus,
    M: shared_bus::BusMutex<Bus = Driver<I2C>>,
{
    pub fn with_mutex(i2c: I2C, a0: bool, a1: bool, a2: bool) -> Self {
        Self(shared_bus::BusMutex::create(Driver::new(
            i2c, true, a0, a1, a2,
        )))
    }

    pub fn split<'a>(&'a mut self) -> Parts<'a, I2C, M> {
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

pub struct Parts<'a, I2C, M = shared_bus::NullMutex<Driver<I2C>>>
where
    I2C: crate::I2cBus,
    M: shared_bus::BusMutex<Bus = Driver<I2C>>,
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
    pub fn new(i2c: I2C, is_a_variant: bool, a0: bool, a1: bool, a2: bool) -> Self {
        let addr = if is_a_variant {
            0x38 | ((a2 as u8) << 2) | ((a1 as u8) << 1) | (a0 as u8)
        } else {
            0x20 | ((a2 as u8) << 2) | ((a1 as u8) << 1) | (a0 as u8)
        };
        Self {
            i2c,
            out: 0xff,
            addr,
        }
    }
}

impl<I2C: crate::I2cBus> crate::PortDriver for Driver<I2C> {
    type Error = I2C::BusError;

    fn set_high(&mut self, mask: u32) -> Result<(), Self::Error> {
        self.out |= mask as u8;
        self.i2c.write(self.addr, &[self.out])?;
        Ok(())
    }
    fn set_low(&mut self, mask: u32) -> Result<(), Self::Error> {
        self.out &= !mask as u8;
        self.i2c.write(self.addr, &[self.out])?;
        Ok(())
    }
    fn is_set_high(&mut self, mask: u32) -> Result<bool, Self::Error> {
        Ok(self.out & mask as u8 != 0)
    }
    fn is_set_low(&mut self, mask: u32) -> Result<bool, Self::Error> {
        Ok(self.out & mask as u8 == 0)
    }

    fn is_high(&mut self, mask: u32) -> Result<bool, Self::Error> {
        let mut buf = [0x00];
        self.i2c.read(self.addr, &mut buf)?;
        Ok(buf[0] & mask as u8 != 0)
    }
    fn is_low(&mut self, mask: u32) -> Result<bool, Self::Error> {
        self.is_high(mask).map(|b| !b)
    }
}

#[cfg(test)]
mod tests {
    use embedded_hal_mock::i2c as mock_i2c;

    #[test]
    fn pcf8574() {
        let expectations = [
            mock_i2c::Transaction::write(0x21, vec![0b11111111]),
            mock_i2c::Transaction::write(0x21, vec![0b11111011]),
            mock_i2c::Transaction::read(0x21, vec![0b01000000]),
            mock_i2c::Transaction::read(0x21, vec![0b10111111]),
        ];
        let mut bus = mock_i2c::Mock::new(&expectations);

        let mut pcf = super::Pcf8574::new(bus.clone(), true, false, false);
        let mut pcf_pins = pcf.split();

        pcf_pins.p2.set_high().unwrap();
        pcf_pins.p2.set_low().unwrap();

        assert!(pcf_pins.p6.is_high().unwrap());
        assert!(pcf_pins.p6.is_low().unwrap());

        bus.done();
    }

    #[test]
    fn pcf8574a() {
        let expectations = [
            mock_i2c::Transaction::write(0x39, vec![0b11111111]),
            mock_i2c::Transaction::write(0x39, vec![0b11101111]),
            mock_i2c::Transaction::read(0x39, vec![0b00000001]),
            mock_i2c::Transaction::read(0x39, vec![0b11111110]),
        ];
        let mut bus = mock_i2c::Mock::new(&expectations);

        let mut pcf_a = super::Pcf8574a::new(bus.clone(), true, false, false);
        let mut pcf_a_pins = pcf_a.split();

        pcf_a_pins.p4.set_high().unwrap();
        pcf_a_pins.p4.set_low().unwrap();

        assert!(pcf_a_pins.p0.is_high().unwrap());
        assert!(pcf_a_pins.p0.is_low().unwrap());

        bus.done();
    }
}
