//! Support for the `TCA6408A` "Remote 8-Bit I2C AND SMBus Low-power I/O Expander  With Interrupt Output, Reset, and Configuration Registers"
use crate::I2cExt;

/// `TCA6408A` "Remote 8-Bit I2C AND SMBus Low-power I/O Expander"
pub struct Tca6408a<M>(M);

impl<I2C> Tca6408a<shared_bus::NullMutex<Driver<I2C>>>
where
    I2C: crate::I2cBus,
{
    pub fn new(i2c: I2C, a0: bool) -> Self {
        Self::with_mutex(i2c, a0)
    }
}

impl<I2C, M> Tca6408a<M>
where
    I2C: crate::I2cBus,
    M: shared_bus::BusMutex<Bus = Driver<I2C>>,
{
    pub fn with_mutex(i2c: I2C, a0: bool) -> Self {
        Self(shared_bus::BusMutex::create(Driver::new(i2c, a0)))
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

pub struct Parts<'a, I2C, M = shared_bus::NullMutex<Driver<I2C>>>
where
    I2C: crate::I2cBus,
    M: shared_bus::BusMutex<Bus = Driver<I2C>>,
{
    pub io0: crate::Pin<'a, crate::mode::Input, M>,
    pub io1: crate::Pin<'a, crate::mode::Input, M>,
    pub io2: crate::Pin<'a, crate::mode::Input, M>,
    pub io3: crate::Pin<'a, crate::mode::Input, M>,
    pub io4: crate::Pin<'a, crate::mode::Input, M>,
    pub io5: crate::Pin<'a, crate::mode::Input, M>,
    pub io6: crate::Pin<'a, crate::mode::Input, M>,
    pub io7: crate::Pin<'a, crate::mode::Input, M>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Regs {
    InputPort = 0x00,
    OutputPort = 0x01,
    PolarityInversion = 0x02,
    Configuration = 0x03,
}

impl From<Regs> for u8 {
    fn from(r: Regs) -> u8 {
        r as u8
    }
}

pub struct Driver<I2C> {
    i2c: I2C,
    addr: u8,
    out: u8,
}

impl<I2C> Driver<I2C> {
    pub fn new(i2c: I2C, a0: bool) -> Self {
        let addr = 0x20 | (a0 as u8);
        Self {
            i2c,
            addr,
            out: 0xff,
        }
    }
}

impl<I2C: crate::I2cBus> crate::PortDriver for Driver<I2C> {
    type Error = I2C::BusError;

    fn set(&mut self, mask_high: u32, mask_low: u32) -> Result<(), Self::Error> {
        let previous = self.out;
        self.out |= mask_high as u8;
        self.out &= !mask_low as u8;
        if self.out != previous {
            self.i2c.write_reg(self.addr, Regs::OutputPort, self.out)
        } else {
            // don't do the transfer when nothing changed
            Ok(())
        }
    }

    fn is_set(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error> {
        Ok(((self.out as u32) & mask_high) | (!(self.out as u32) & mask_low))
    }

    fn get(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error> {
        let in_ = self.i2c.read_reg(self.addr, Regs::InputPort)? as u32;
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
            crate::Direction::Input => (mask as u8, 0),
            crate::Direction::Output => (0, mask as u8),
        };
        self.i2c
            .update_reg(self.addr, Regs::Configuration, mask_set, mask_clear)
    }
}

impl<I2C: crate::I2cBus> crate::PortDriverPolarity for Driver<I2C> {
    fn set_polarity(&mut self, mask: u32, inverted: bool) -> Result<(), Self::Error> {
        let (mask_set, mask_clear) = match inverted {
            false => (0, mask as u8),
            true => (mask as u8, 0),
        };

        self.i2c
            .update_reg(self.addr, Regs::PolarityInversion, mask_set, mask_clear)
    }
}

#[cfg(test)]
mod tests {
    use embedded_hal_mock::eh1::i2c as mock_i2c;

    #[test]
    fn tca6408a() {
        let expectations = [
            // pin setup io0
            mock_i2c::Transaction::write(0x21, vec![0x01, 0xfe]),
            mock_i2c::Transaction::write_read(0x21, vec![0x03], vec![0xff]),
            mock_i2c::Transaction::write(0x21, vec![0x03, 0xfe]),
            // pin setup io1
            mock_i2c::Transaction::write_read(0x21, vec![0x03], vec![0xfe]),
            mock_i2c::Transaction::write(0x21, vec![0x03, 0xfc]),
            // pin setup io0 as input
            mock_i2c::Transaction::write_read(0x21, vec![0x03], vec![0xfc]),
            mock_i2c::Transaction::write(0x21, vec![0x03, 0xfd]),
            // io1 writes
            mock_i2c::Transaction::write(0x21, vec![0x01, 0xfc]),
            mock_i2c::Transaction::write(0x21, vec![0x01, 0xfe]),
            mock_i2c::Transaction::write(0x21, vec![0x01, 0xfc]),
            // io0 reads
            mock_i2c::Transaction::write_read(0x21, vec![0x00], vec![0x01]),
            mock_i2c::Transaction::write_read(0x21, vec![0x00], vec![0x00]),
            // io4 polarity
            mock_i2c::Transaction::write_read(0x21, vec![0x02], vec![0x00]),
            mock_i2c::Transaction::write(0x21, vec![0x02, 0x10]),
            // io5 polarity
            mock_i2c::Transaction::write_read(0x21, vec![0x02], vec![0x10]),
            mock_i2c::Transaction::write(0x21, vec![0x02, 0x30]),
            mock_i2c::Transaction::write_read(0x21, vec![0x02], vec![0x30]),
            mock_i2c::Transaction::write(0x21, vec![0x02, 0x10]),
        ];
        let mut bus = mock_i2c::Mock::new(&expectations);

        let mut pca = super::Tca6408a::new(bus.clone(), true);
        let pca_pins = pca.split();

        let io0 = pca_pins.io0.into_output().unwrap();
        let mut io1 = pca_pins.io1.into_output_high().unwrap();

        let io0 = io0.into_input().unwrap();

        io1.set_low().unwrap();
        io1.set_high().unwrap();
        io1.toggle().unwrap();

        assert!(io0.is_high().unwrap());
        assert!(io0.is_low().unwrap());

        pca_pins.io4.into_inverted().unwrap();
        let mut io5 = pca_pins.io5;
        io5.set_inverted(true).unwrap();
        io5.set_inverted(false).unwrap();

        bus.done();
    }
}
