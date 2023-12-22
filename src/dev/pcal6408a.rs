//! Support for the `PCAL6408A` "8-bit I2C-bus and SMBus I/O port with interrupt"
use crate::I2cExt;

/// `PCAL6408A` "8-bit I2C-bus and SMBus I/O port with interrupt"
pub struct Pcal6408a<M>(M);

impl<I2C> Pcal6408a<shared_bus::NullMutex<Driver<I2C>>>
where
    I2C: crate::I2cBus,
{
    pub fn new(i2c: I2C, addr: bool) -> Self {
        Self::with_mutex(i2c, addr)
    }
}

impl<I2C, M> Pcal6408a<M>
where
    I2C: crate::I2cBus,
    M: shared_bus::BusMutex<Bus = Driver<I2C>>,
{
    pub fn with_mutex(i2c: I2C, addr: bool) -> Self {
        Self(shared_bus::BusMutex::create(Driver::new(i2c, addr)))
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
    OutputDriveStrength0 = 0x40,
    OutputDriveStrength1 = 0x41,
    InputLatch = 0x42,
    PullEnable = 0x43,
    PullSelection = 0x44,
    InterruptMask = 0x45,
    InterruptStatus = 0x46,
    OutputPortConfiguration = 0x47, // Bit 0: Push-Pull (0) or Open-Drain (1) for all Outputs
}

impl From<Regs> for u8 {
    fn from(r: Regs) -> u8 {
        r as u8
    }
}

pub struct Driver<I2C> {
    i2c: I2C,
    out: Option<u8>,
    addr: u8,
}

impl<I2C> Driver<I2C> {
    pub fn new(i2c: I2C, addr: bool) -> Self {
        let addr = 0x20 | (addr as u8);
        Self {
            i2c,
            out: None,
            addr,
        }
    }
}

impl<I2C: crate::I2cBus> Driver<I2C> {
    fn get_out(&mut self) -> Result<u8, I2C::BusError> {
        // Make sure the state of the OutputPort register is actually known instead of assumed to avoid glitches on reboot.
        // This is necessary because the OutputPort register is written instead of updated.
        match self.out {
            Some(out) => Ok(out),
            None => {
                let out = self.i2c.read_reg(self.addr, Regs::OutputPort)?;
                self.out = Some(out);
                Ok(out)
            }
        }
    }
}

impl<I2C: crate::I2cBus> crate::PortDriver for Driver<I2C> {
    type Error = I2C::BusError;

    fn set(&mut self, mask_high: u32, mask_low: u32) -> Result<(), Self::Error> {
        let mut out = self.get_out()?;
        out |= mask_high as u8;
        out &= !mask_low as u8;
        self.out = Some(out);
        if (mask_high | mask_low) & 0x00FF != 0 {
            self.i2c
                .write_reg(self.addr, Regs::OutputPort, (out & 0xFF) as u8)?;
        }
        Ok(())
    }

    fn is_set(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error> {
        let out = self.get_out()?;
        Ok(((out as u32) & mask_high) | (!(out as u32) & mask_low))
    }

    fn get(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error> {
        let io0 = if (mask_high | mask_low) & 0x00FF != 0 {
            self.i2c.read_reg(self.addr, Regs::InputPort)?
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
        if mask & 0xFF == 0 {
            return Ok(());
        }
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
            .update_reg(self.addr, Regs::Configuration, mask_set, mask_clear)?;
        Ok(())
    }
}

impl<I2C: crate::I2cBus> crate::PortDriverPolarity for Driver<I2C> {
    fn set_polarity(&mut self, mask: u32, inverted: bool) -> Result<(), Self::Error> {
        if mask & 0xFF == 0 {
            return Ok(());
        }
        let (mask_set, mask_clear) = match inverted {
            false => (0, mask as u8),
            true => (mask as u8, 0),
        };

        self.i2c
            .update_reg(self.addr, Regs::PolarityInversion, mask_set, mask_clear)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use embedded_hal_mock::i2c as mock_i2c;

    #[test]
    fn pca6408a() {
        let expectations = [
            // pin setup io0
            mock_i2c::Transaction::write_read(0x21, vec![0x01], vec![0xff]),
            mock_i2c::Transaction::write(0x21, vec![0x01, 0xfe]),
            mock_i2c::Transaction::write_read(0x21, vec![0x03], vec![0xff]),
            mock_i2c::Transaction::write(0x21, vec![0x03, 0xfe]),
            // pin setup io7
            mock_i2c::Transaction::write(0x21, vec![0x01, 0x7e]),
            mock_i2c::Transaction::write_read(0x21, vec![0x03], vec![0xfe]),
            mock_i2c::Transaction::write(0x21, vec![0x03, 0x7e]),
            mock_i2c::Transaction::write_read(0x21, vec![0x03], vec![0x7e]),
            mock_i2c::Transaction::write(0x21, vec![0x03, 0xfe]),
            // output io0
            mock_i2c::Transaction::write(0x21, vec![0x01, 0x7f]),
            mock_i2c::Transaction::write(0x21, vec![0x01, 0x7e]),
            // input io7
            mock_i2c::Transaction::write_read(0x21, vec![0x00], vec![0x80]),
            mock_i2c::Transaction::write_read(0x21, vec![0x00], vec![0x7f]),
            // polarity io7
            mock_i2c::Transaction::write_read(0x21, vec![0x02], vec![0x00]),
            mock_i2c::Transaction::write(0x21, vec![0x02, 0x80]),
            mock_i2c::Transaction::write_read(0x21, vec![0x02], vec![0xff]),
            mock_i2c::Transaction::write(0x21, vec![0x02, 0x7f]),
        ];
        let mut bus = mock_i2c::Mock::new(&expectations);

        let mut pcal = super::Pcal6408a::new(bus.clone(), true);
        let pcal_pins = pcal.split();

        let mut io0 = pcal_pins.io0.into_output().unwrap();
        let io7 = pcal_pins.io7.into_output().unwrap();
        let io7 = io7.into_input().unwrap();

        // output high and low
        io0.set_high().unwrap();
        io0.set_low().unwrap();

        // input high and low
        assert!(io7.is_high().unwrap());
        assert!(io7.is_low().unwrap());

        let mut io7 = io7.into_inverted().unwrap();
        io7.set_inverted(false).unwrap();

        bus.done();
    }
}
