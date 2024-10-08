//! Support for the `PCA9554` and `PCA9554a` "8-bit I2C-bus and SMBus I/O port with interrupt"
use crate::I2cExt;

/// `PCA9554` "8-bit I2C-bus and SMBus I/O port with interrupt"
pub struct Pca9554<M>(M);
/// `PCA9554A` "8-bit I2C-bus and SMBus I/O port with interrupt"
pub struct Pca9554A<M>(M);

impl<I2C> Pca9554<core::cell::RefCell<Driver<I2C>>>
where
    I2C: crate::I2cBus,
{
    pub fn new(i2c: I2C, a0: bool, a1: bool, a2: bool) -> Self {
        Self::with_mutex(i2c, a0, a1, a2)
    }
}

impl<I2C> Pca9554A<core::cell::RefCell<Driver<I2C>>>
where
    I2C: crate::I2cBus,
{
    pub fn new(i2c: I2C, a0: bool, a1: bool, a2: bool) -> Self {
        Self::with_mutex(i2c, a0, a1, a2)
    }
}

impl<I2C, M> Pca9554<M>
where
    I2C: crate::I2cBus,
    M: crate::PortMutex<Port = Driver<I2C>>,
{
    pub fn with_mutex(i2c: I2C, a0: bool, a1: bool, a2: bool) -> Self {
        Self(crate::PortMutex::create(Driver::new(
            i2c, false, a0, a1, a2,
        )))
    }

    pub fn split<'a>(&'a mut self) -> Parts<'a, I2C, M> {
        Parts {
            io0: crate::Pin::new(0, &self.0),
            io1: crate::Pin::new(1, &self.0),
            io2: crate::Pin::new(2, &self.0),
            io3: crate::Pin::new(3, &self.0),
            io4: crate::Pin::new(4, &self.0),
            io5: crate::Pin::new(5, &self.0),
            io6: crate::Pin::new(6, &self.0),
            io7: crate::Pin::new(7, &self.0),
        }
    }
}

impl<I2C, M> Pca9554A<M>
where
    I2C: crate::I2cBus,
    M: crate::PortMutex<Port = Driver<I2C>>,
{
    pub fn with_mutex(i2c: I2C, a0: bool, a1: bool, a2: bool) -> Self {
        Self(crate::PortMutex::create(Driver::new(i2c, true, a0, a1, a2)))
    }

    pub fn split(&mut self) -> Parts<'_, I2C, M> {
        Parts {
            io0: crate::Pin::new(0, &self.0),
            io1: crate::Pin::new(1, &self.0),
            io2: crate::Pin::new(2, &self.0),
            io3: crate::Pin::new(3, &self.0),
            io4: crate::Pin::new(4, &self.0),
            io5: crate::Pin::new(5, &self.0),
            io6: crate::Pin::new(6, &self.0),
            io7: crate::Pin::new(7, &self.0),
        }
    }
}

pub struct Parts<'a, I2C, M = core::cell::RefCell<Driver<I2C>>>
where
    I2C: crate::I2cBus,
    M: crate::PortMutex<Port = Driver<I2C>>,
{
    pub io0: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub io1: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub io2: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub io3: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub io4: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub io5: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub io6: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
    pub io7: crate::Pin<'a, crate::mode::QuasiBidirectional, M>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Regs {
    InputPort0 = 0x00,
    OutputPort0 = 0x01,
    PolarityInversion0 = 0x02,
    Configuration0 = 0x03,
}

impl From<Regs> for u8 {
    fn from(r: Regs) -> u8 {
        r as u8
    }
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

    fn set(&mut self, mask_high: u32, mask_low: u32) -> Result<(), Self::Error> {
        self.out |= mask_high as u8;
        self.out &= !mask_low as u8;
        self.i2c
            .write_reg(self.addr, Regs::OutputPort0, (self.out & 0xFF) as u8)?;
        Ok(())
    }

    fn is_set(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error> {
        Ok(((self.out as u32) & mask_high) | (!(self.out as u32) & mask_low))
    }

    fn get(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error> {
        let io0 = if (mask_high | mask_low) & 0x00FF != 0 {
            self.i2c.read_reg(self.addr, Regs::InputPort0)?
        } else {
            0
        };
        let in_ = io0 as u32;
        Ok((in_ & mask_high) | (!in_ & mask_low))
    }
}

impl<I2C: crate::I2cBus> crate::PortDriverTotemPole for Driver<I2C> {
    fn set_direction(
        &mut self,
        mask: u32,
        dir: crate::Direction,
        state: bool,
    ) -> Result<(), Self::Error> {
        // set state before switching direction to prevent glitch
        if dir == crate::Direction::Output {
            use crate::PortDriver;
            if state {
                self.set(mask, 0)?;
            } else {
                self.set(0, mask)?;
            }
        }

        let (mask_set, mask_clear) = match dir {
            crate::Direction::Input => (mask as u16, 0),
            crate::Direction::Output => (0, mask as u16),
        };
        if mask & 0x00FF != 0 {
            self.i2c.update_reg(
                self.addr,
                Regs::Configuration0,
                (mask_set & 0xFF) as u8,
                (mask_clear & 0xFF) as u8,
            )?;
        }
        Ok(())
    }
}

impl<I2C: crate::I2cBus> crate::PortDriverPolarity for Driver<I2C> {
    fn set_polarity(&mut self, mask: u32, inverted: bool) -> Result<(), Self::Error> {
        let (mask_set, mask_clear) = match inverted {
            false => (0, mask as u8),
            true => (mask as u8, 0),
        };

        self.i2c.update_reg(
            self.addr,
            Regs::PolarityInversion0,
            (mask_set & 0xFF) as u8,
            (mask_clear & 0xFF) as u8,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use embedded_hal_mock::eh1::i2c as mock_i2c;

    #[test]
    fn pca9554a() {
        let expectations = [
            // set pin0 low and then high
            mock_i2c::Transaction::write(0x39, vec![0x01, 0xfe]),
            mock_i2c::Transaction::write(0x39, vec![0x01, 0xff]),
            mock_i2c::Transaction::write_read(0x39, vec![0x00], vec![0xff]),
            // set pin1 low
            mock_i2c::Transaction::write(0x39, vec![0x01, 0xfd]),
            mock_i2c::Transaction::write_read(0x39, vec![0x00], vec![0xfd]),
            // set pin2 as output
            mock_i2c::Transaction::write(0x39, vec![0x01, 0xf9]),
            mock_i2c::Transaction::write_read(0x39, vec![0x03], vec![0xff]),
            mock_i2c::Transaction::write(0x39, vec![0x03, 0xfb]),
        ];
        let mut bus = mock_i2c::Mock::new(&expectations);

        let mut pca = super::Pca9554A::new(bus.clone(), true, false, false);
        let pca_pins = pca.split();

        let mut pin0 = pca_pins.io0;
        let mut pin1 = pca_pins.io1;
        let pin2 = pca_pins.io2;

        pin0.set_low().unwrap();
        pin0.set_high().unwrap();
        assert!(pin0.is_high().unwrap());
        pin1.set_low().unwrap();
        assert!(pin1.is_low().unwrap());
        pin2.into_output().unwrap();

        bus.done();
    }

    #[test]
    fn pca9554() {
        let expectations = [
            // set pin0 low and then high
            mock_i2c::Transaction::write(0x21, vec![0x01, 0xfe]),
            mock_i2c::Transaction::write(0x21, vec![0x01, 0xff]),
            mock_i2c::Transaction::write_read(0x21, vec![0x00], vec![0xff]),
            // set pin1 low
            mock_i2c::Transaction::write(0x21, vec![0x01, 0xfd]),
            mock_i2c::Transaction::write_read(0x21, vec![0x00], vec![0xfd]),
            // set pin2 as output
            mock_i2c::Transaction::write(0x21, vec![0x01, 0xf9]),
            mock_i2c::Transaction::write_read(0x21, vec![0x03], vec![0xff]),
            mock_i2c::Transaction::write(0x21, vec![0x03, 0xfb]),
        ];
        let mut bus = mock_i2c::Mock::new(&expectations);

        let mut pca = super::Pca9554::new(bus.clone(), true, false, false);
        let pca_pins = pca.split();

        let mut pin0 = pca_pins.io0;
        let mut pin1 = pca_pins.io1;
        let pin2 = pca_pins.io2;

        pin0.set_low().unwrap();
        pin0.set_high().unwrap();
        assert!(pin0.is_high().unwrap());
        pin1.set_low().unwrap();
        assert!(pin1.is_low().unwrap());
        pin2.into_output().unwrap();

        bus.done();
    }
}
