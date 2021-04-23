use crate::I2cExt;
use core::marker::PhantomData;

pub trait Regs8 {
    const INPUT: u8;
    const OUTPUT: u8;
    const POLARITY: u8;
    const CONFIGURATION: u8;
}

pub struct Regs8Default;

impl Regs8 for Regs8Default {
    const INPUT: u8 = 0x00;
    const OUTPUT: u8 = 0x01;
    const POLARITY: u8 = 0x02;
    const CONFIGURATION: u8 = 0x03;
}

pub struct Driver8<I2C, REGS, const ADDRESS: u8> {
    i2c: I2C,
    output_state: u8,
    _r: PhantomData<REGS>,
}

impl<I2C, REGS, const ADDRESS: u8> Driver8<I2C, REGS, ADDRESS> {
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            output_state: 0xff,
            _r: PhantomData,
        }
    }
}

impl<I2C, REGS, const ADDRESS: u8> crate::PortDriver for Driver8<I2C, REGS, ADDRESS>
where
    I2C: crate::I2cBus,
    REGS: Regs8,
{
    type Error = I2C::BusError;

    fn set_high(&mut self, mask: u32) -> Result<(), Self::Error> {
        self.output_state |= mask as u8;
        self.i2c.write_reg(ADDRESS, REGS::OUTPUT, self.output_state)
    }
    fn set_low(&mut self, mask: u32) -> Result<(), Self::Error> {
        self.output_state &= !mask as u8;
        self.i2c.write_reg(ADDRESS, REGS::OUTPUT, self.output_state)
    }
    fn is_set_high(&mut self, mask: u32) -> Result<bool, Self::Error> {
        Ok(self.output_state & mask as u8 != 0)
    }
    fn is_set_low(&mut self, mask: u32) -> Result<bool, Self::Error> {
        Ok(self.output_state & mask as u8 == 0)
    }

    fn is_high(&mut self, mask: u32) -> Result<bool, Self::Error> {
        Ok(self.i2c.read_reg(ADDRESS, REGS::INPUT)? & mask as u8 != 0)
    }
    fn is_low(&mut self, mask: u32) -> Result<bool, Self::Error> {
        self.is_high(mask).map(|b| !b)
    }

    fn set_direction(&mut self, mask: u32, dir: crate::Direction) -> Result<(), Self::Error> {
        let (mask_set, mask_clear) = match dir {
            crate::Direction::Input => (mask as u8, 0),
            crate::Direction::Output => (0, mask as u8),
        };
        self.i2c
            .update_reg(ADDRESS, REGS::CONFIGURATION, mask_set, mask_clear)
    }
}
