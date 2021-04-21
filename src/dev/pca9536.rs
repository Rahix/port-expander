//! Support for the PCA9536 "4-bit I2C-bus and SMBus I/O port"

pub struct Pca9536<I2C, M = shared_bus::NullMutex<Driver<I2C>>>
where
    M: shared_bus::BusMutex<Bus = Driver<I2C>>,
{
    inner: M,
}

impl<I2C, M> Pca9536<I2C, M>
where
    I2C: crate::I2cBus,
    M: shared_bus::BusMutex<Bus = Driver<I2C>>,
{
    pub fn new(i2c: I2C) -> Self {
        Self {
            inner: shared_bus::BusMutex::create(Driver::new(i2c)),
        }
    }

    pub fn split<'b>(&'b mut self) -> Parts<'b, I2C, M> {
        Parts {
            io0: crate::Pin::new(0, &self.inner),
            io1: crate::Pin::new(1, &self.inner),
            io2: crate::Pin::new(2, &self.inner),
            io3: crate::Pin::new(3, &self.inner),
        }
    }
}

impl<I2C, M> crate::Port for Pca9536<I2C, M>
where
    I2C: crate::I2cBus,
    M: shared_bus::BusMutex<Bus = Driver<I2C>>,
{
    type Driver = Driver<I2C>;
}

pub struct Parts<'a, I2C, M = shared_bus::NullMutex<Driver<I2C>>>
where
    I2C: crate::I2cBus,
    M: shared_bus::BusMutex<Bus = Driver<I2C>>,
{
    pub io0: crate::Pin<'a, crate::mode::Input, M, Pca9536<I2C, M>>,
    pub io1: crate::Pin<'a, crate::mode::Input, M, Pca9536<I2C, M>>,
    pub io2: crate::Pin<'a, crate::mode::Input, M, Pca9536<I2C, M>>,
    pub io3: crate::Pin<'a, crate::mode::Input, M, Pca9536<I2C, M>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Regs {
    InputPort = 0x00,
    OutputPort = 0x01,
    PolarityInversion = 0x02,
    Configuration = 0x03,
}

const ADDRESS: u8 = 0x41;

pub struct Driver<I2C> {
    i2c: I2C,
    out: u8,
}

impl<I2C> Driver<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c, out: 0xff }
    }
}

impl<I2C: crate::I2cBus> crate::PortDriver for Driver<I2C> {
    type Error = I2C::BusError;

    fn set_high(&mut self, mask: u32) -> Result<(), Self::Error> {
        self.out |= mask as u8;
        self.i2c.write(ADDRESS, &[Regs::OutputPort as u8, self.out])?;
        Ok(())
    }
    fn set_low(&mut self, mask: u32) -> Result<(), Self::Error> {
        self.out &= !mask as u8;
        self.i2c.write(ADDRESS, &[Regs::OutputPort as u8, self.out])?;
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
        self.i2c.write_read(ADDRESS, &[Regs::InputPort as u8], &mut buf)?;
        Ok(buf[0] & mask as u8 != 0)
    }
    fn is_low(&mut self, mask: u32) -> Result<bool, Self::Error> {
        self.is_high(mask).map(|b| !b)
    }

    fn set_direction(&mut self, mask: u32, dir: crate::Direction) -> Result<(), Self::Error> {
        let mut buf = [0x00];
        self.i2c.write_read(ADDRESS, &[Regs::Configuration as u8], &mut buf)?;
        match dir {
            crate::Direction::Input => buf[0] |= mask as u8,
            crate::Direction::Output => buf[0] &= !mask as u8,
        }
        self.i2c.write(ADDRESS, &[Regs::Configuration as u8, buf[0]])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use embedded_hal_mock::i2c as mock_i2c;

    #[test]
    fn pca9536() {
        let expectations = [
            // pin setup io0
            mock_i2c::Transaction::write_read(super::ADDRESS, vec![0x03], vec![0xff]),
            mock_i2c::Transaction::write(super::ADDRESS, vec![0x03, 0xfe]),
            // pin setup io1
            mock_i2c::Transaction::write_read(super::ADDRESS, vec![0x03], vec![0xfe]),
            mock_i2c::Transaction::write(super::ADDRESS, vec![0x03, 0xfc]),
            // pin setup io0 as input
            mock_i2c::Transaction::write_read(super::ADDRESS, vec![0x03], vec![0xfc]),
            mock_i2c::Transaction::write(super::ADDRESS, vec![0x03, 0xfd]),
            // io1 writes
            mock_i2c::Transaction::write(super::ADDRESS, vec![0x01, 0xfd]),
            mock_i2c::Transaction::write(super::ADDRESS, vec![0x01, 0xff]),
            mock_i2c::Transaction::write(super::ADDRESS, vec![0x01, 0xfd]),
            // io0 reads
            mock_i2c::Transaction::write_read(super::ADDRESS, vec![0x00], vec![0x01]),
            mock_i2c::Transaction::write_read(super::ADDRESS, vec![0x00], vec![0x00]),
        ];
        let mut bus = mock_i2c::Mock::new(&expectations);

        let mut pca = super::Pca9536::<_, shared_bus::NullMutex<_>>::new(bus.clone());
        let pca_pins = pca.split();

        let io0 = pca_pins.io0.into_output().unwrap();
        let io1 = pca_pins.io1.into_output().unwrap();

        let io0 = io0.into_input().unwrap();

        io1.set_low().unwrap();
        io1.set_high().unwrap();
        io1.toggle().unwrap();

        assert!(io0.is_high().unwrap());
        assert!(io0.is_low().unwrap());

        bus.done();
    }
}
