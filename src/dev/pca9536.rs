//! Support for the PCA9536 "4-bit I2C-bus and SMBus I/O port"

pub struct Pca9536<M>(M);

impl<I2C> Pca9536<shared_bus::NullMutex<Driver<I2C>>>
where
    I2C: crate::I2cBus,
{
    pub fn new(i2c: I2C) -> Self {
        Self(shared_bus::BusMutex::create(Driver::new(i2c)))
    }
}

impl<I2C, M> Pca9536<M>
where
    I2C: crate::I2cBus,
    M: shared_bus::BusMutex<Bus = Driver<I2C>>,
{
    pub fn with_mutex(i2c: I2C) -> Self {
        Self(shared_bus::BusMutex::create(Driver::new(i2c)))
    }

    pub fn split<'a>(&'a mut self) -> Parts<'a, I2C, M> {
        Parts {
            io0: crate::Pin::new(0, &self.0),
            io1: crate::Pin::new(1, &self.0),
            io2: crate::Pin::new(2, &self.0),
            io3: crate::Pin::new(3, &self.0),
        }
    }
}

pub struct Parts<'a, I2C, M = shared_bus::NullMutex<Driver<I2C>>>
where
    I2C: crate::I2cBus,
    M: shared_bus::BusMutex<Bus = Driver<I2C>>,
{
    pub io0: crate::Pin<'a, crate::mode::Input, M>,
    pub io1: crate::Pin<'a, crate::mode::Input, M>,
    pub io2: crate::Pin<'a, crate::mode::Input, M>,
    pub io3: crate::Pin<'a, crate::mode::Input, M>,
}

const ADDRESS: u8 = 0x41;

pub type Driver<I2C> = crate::driver::Driver8<I2C, ADDRESS>;

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

        let mut pca = super::Pca9536::new(bus.clone());
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
