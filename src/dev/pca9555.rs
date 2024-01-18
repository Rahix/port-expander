//! Support for the `PCA9555` "16-bit I2C-bus and SMBus I/O port with interrupt"
use crate::I2cExt;

/// `PCA9555` "16-bit I2C-bus and SMBus I/O port with interrupt"
pub struct Pca9555<M>(M);

impl<I2C> Pca9555<shared_bus::NullMutex<Driver<I2C>>>
where
    I2C: crate::I2cBus,
{
    pub fn new(i2c: I2C, a0: bool, a1: bool, a2: bool) -> Self {
        Self::with_mutex(i2c, a0, a1, a2)
    }
}

impl<I2C, M> Pca9555<M>
where
    I2C: crate::I2cBus,
    M: shared_bus::BusMutex<Bus = Driver<I2C>>,
{
    pub fn with_mutex(i2c: I2C, a0: bool, a1: bool, a2: bool) -> Self {
        Self(shared_bus::BusMutex::create(Driver::new(i2c, a0, a1, a2)))
    }

    pub fn split(&mut self) -> Parts<'_, I2C, M> {
        Parts {
            io0_0: crate::Pin::new(0, &self.0),
            io0_1: crate::Pin::new(1, &self.0),
            io0_2: crate::Pin::new(2, &self.0),
            io0_3: crate::Pin::new(3, &self.0),
            io0_4: crate::Pin::new(4, &self.0),
            io0_5: crate::Pin::new(5, &self.0),
            io0_6: crate::Pin::new(6, &self.0),
            io0_7: crate::Pin::new(7, &self.0),
            io1_0: crate::Pin::new(8, &self.0),
            io1_1: crate::Pin::new(9, &self.0),
            io1_2: crate::Pin::new(10, &self.0),
            io1_3: crate::Pin::new(11, &self.0),
            io1_4: crate::Pin::new(12, &self.0),
            io1_5: crate::Pin::new(13, &self.0),
            io1_6: crate::Pin::new(14, &self.0),
            io1_7: crate::Pin::new(15, &self.0),
        }
    }
}

pub struct Parts<'a, I2C, M = shared_bus::NullMutex<Driver<I2C>>>
where
    I2C: crate::I2cBus,
    M: shared_bus::BusMutex<Bus = Driver<I2C>>,
{
    pub io0_0: crate::Pin<'a, crate::mode::Input, M>,
    pub io0_1: crate::Pin<'a, crate::mode::Input, M>,
    pub io0_2: crate::Pin<'a, crate::mode::Input, M>,
    pub io0_3: crate::Pin<'a, crate::mode::Input, M>,
    pub io0_4: crate::Pin<'a, crate::mode::Input, M>,
    pub io0_5: crate::Pin<'a, crate::mode::Input, M>,
    pub io0_6: crate::Pin<'a, crate::mode::Input, M>,
    pub io0_7: crate::Pin<'a, crate::mode::Input, M>,
    pub io1_0: crate::Pin<'a, crate::mode::Input, M>,
    pub io1_1: crate::Pin<'a, crate::mode::Input, M>,
    pub io1_2: crate::Pin<'a, crate::mode::Input, M>,
    pub io1_3: crate::Pin<'a, crate::mode::Input, M>,
    pub io1_4: crate::Pin<'a, crate::mode::Input, M>,
    pub io1_5: crate::Pin<'a, crate::mode::Input, M>,
    pub io1_6: crate::Pin<'a, crate::mode::Input, M>,
    pub io1_7: crate::Pin<'a, crate::mode::Input, M>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Regs {
    InputPort0 = 0x00,
    InputPort1 = 0x01,
    OutputPort0 = 0x02,
    OutputPort1 = 0x03,
    PolarityInversion0 = 0x04,
    PolarityInversion1 = 0x05,
    Configuration0 = 0x06,
    Configuration1 = 0x07,
}

impl From<Regs> for u8 {
    fn from(r: Regs) -> u8 {
        r as u8
    }
}

pub struct Driver<I2C> {
    i2c: I2C,
    out: u16,
    addr: u8,
}

impl<I2C> Driver<I2C> {
    pub fn new(i2c: I2C, a0: bool, a1: bool, a2: bool) -> Self {
        let addr = 0x20 | ((a2 as u8) << 2) | ((a1 as u8) << 1) | (a0 as u8);
        Self {
            i2c,
            out: 0xffff,
            addr,
        }
    }
}

impl<I2C: crate::I2cBus> crate::PortDriver for Driver<I2C> {
    type Error = I2C::BusError;

    fn set(&mut self, mask_high: u32, mask_low: u32) -> Result<(), Self::Error> {
        self.out |= mask_high as u16;
        self.out &= !mask_low as u16;
        if (mask_high | mask_low) & 0x00FF != 0 {
            self.i2c
                .write_reg(self.addr, Regs::OutputPort0, (self.out & 0xFF) as u8)?;
        }
        if (mask_high | mask_low) & 0xFF00 != 0 {
            self.i2c
                .write_reg(self.addr, Regs::OutputPort1, (self.out >> 8) as u8)?;
        }
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
        let io1 = if (mask_high | mask_low) & 0xFF00 != 0 {
            self.i2c.read_reg(self.addr, Regs::InputPort1)?
        } else {
            0
        };
        let in_ = ((io1 as u32) << 8) | io0 as u32;
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
        if mask & 0xFF00 != 0 {
            self.i2c.update_reg(
                self.addr,
                Regs::Configuration1,
                (mask_set >> 8) as u8,
                (mask_clear >> 8) as u8,
            )?;
        }
        Ok(())
    }
}

impl<I2C: crate::I2cBus> crate::PortDriverPolarity for Driver<I2C> {
    fn set_polarity(&mut self, mask: u32, inverted: bool) -> Result<(), Self::Error> {
        let (mask_set, mask_clear) = match inverted {
            false => (0, mask as u16),
            true => (mask as u16, 0),
        };

        if mask & 0x00FF != 0 {
            self.i2c.update_reg(
                self.addr,
                Regs::PolarityInversion0,
                (mask_set & 0xFF) as u8,
                (mask_clear & 0xFF) as u8,
            )?;
        }
        if mask & 0xFF00 != 0 {
            self.i2c.update_reg(
                self.addr,
                Regs::PolarityInversion1,
                (mask_set >> 8) as u8,
                (mask_clear >> 8) as u8,
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use embedded_hal_mock::eh1::i2c as mock_i2c;

    #[test]
    fn pca9555() {
        let expectations = [
            // pin setup io0_0
            mock_i2c::Transaction::write(0x22, vec![0x02, 0xfe]),
            mock_i2c::Transaction::write_read(0x22, vec![0x06], vec![0xff]),
            mock_i2c::Transaction::write(0x22, vec![0x06, 0xfe]),
            // pin setup io0_7
            mock_i2c::Transaction::write(0x22, vec![0x02, 0x7e]),
            mock_i2c::Transaction::write_read(0x22, vec![0x06], vec![0xfe]),
            mock_i2c::Transaction::write(0x22, vec![0x06, 0x7e]),
            mock_i2c::Transaction::write_read(0x22, vec![0x06], vec![0x7e]),
            mock_i2c::Transaction::write(0x22, vec![0x06, 0xfe]),
            // pin setup io1_0
            mock_i2c::Transaction::write(0x22, vec![0x03, 0xfe]),
            mock_i2c::Transaction::write_read(0x22, vec![0x07], vec![0xff]),
            mock_i2c::Transaction::write(0x22, vec![0x07, 0xfe]),
            // pin setup io1_7
            mock_i2c::Transaction::write(0x22, vec![0x03, 0x7e]),
            mock_i2c::Transaction::write_read(0x22, vec![0x07], vec![0xfe]),
            mock_i2c::Transaction::write(0x22, vec![0x07, 0x7e]),
            mock_i2c::Transaction::write_read(0x22, vec![0x07], vec![0x7e]),
            mock_i2c::Transaction::write(0x22, vec![0x07, 0xfe]),
            // output io0_0, io1_0
            mock_i2c::Transaction::write(0x22, vec![0x02, 0x7f]),
            mock_i2c::Transaction::write(0x22, vec![0x02, 0x7e]),
            mock_i2c::Transaction::write(0x22, vec![0x03, 0x7f]),
            mock_i2c::Transaction::write(0x22, vec![0x03, 0x7e]),
            // input io0_7, io1_7
            mock_i2c::Transaction::write_read(0x22, vec![0x00], vec![0x80]),
            mock_i2c::Transaction::write_read(0x22, vec![0x00], vec![0x7f]),
            mock_i2c::Transaction::write_read(0x22, vec![0x01], vec![0x80]),
            mock_i2c::Transaction::write_read(0x22, vec![0x01], vec![0x7f]),
            // polarity io0_7, io1_7
            mock_i2c::Transaction::write_read(0x22, vec![0x04], vec![0x00]),
            mock_i2c::Transaction::write(0x22, vec![0x04, 0x80]),
            mock_i2c::Transaction::write_read(0x22, vec![0x04], vec![0xff]),
            mock_i2c::Transaction::write(0x22, vec![0x04, 0x7f]),
            mock_i2c::Transaction::write_read(0x22, vec![0x05], vec![0x00]),
            mock_i2c::Transaction::write(0x22, vec![0x05, 0x80]),
            mock_i2c::Transaction::write_read(0x22, vec![0x05], vec![0xff]),
            mock_i2c::Transaction::write(0x22, vec![0x05, 0x7f]),
        ];
        let mut bus = mock_i2c::Mock::new(&expectations);

        let mut pca = super::Pca9555::new(bus.clone(), false, true, false);
        let pca_pins = pca.split();

        let mut io0_0 = pca_pins.io0_0.into_output().unwrap();
        let io0_7 = pca_pins.io0_7.into_output().unwrap();
        let io0_7 = io0_7.into_input().unwrap();

        let mut io1_0 = pca_pins.io1_0.into_output().unwrap();
        let io1_7 = pca_pins.io1_7.into_output().unwrap();
        let io1_7 = io1_7.into_input().unwrap();

        // output high and low
        io0_0.set_high().unwrap();
        io0_0.set_low().unwrap();
        io1_0.set_high().unwrap();
        io1_0.set_low().unwrap();

        // input high and low
        assert!(io0_7.is_high().unwrap());
        assert!(io0_7.is_low().unwrap());
        assert!(io1_7.is_high().unwrap());
        assert!(io1_7.is_low().unwrap());

        let mut io0_7 = io0_7.into_inverted().unwrap();
        io0_7.set_inverted(false).unwrap();
        let mut io1_7 = io1_7.into_inverted().unwrap();
        io1_7.set_inverted(false).unwrap();

        bus.done();
    }
}
