use crate::I2cExt;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Regs8 {
    InputPort = 0x00,
    OutputPort = 0x01,
    PolarityInversion = 0x02,
    Configuration = 0x03,
}

impl From<Regs8> for u8 {
    fn from(r: Regs8) -> u8 {
        r as u8
    }
}

pub struct Driver8<I2C, const ADDRESS: u8> {
    i2c: I2C,
    output_state: u8,
}

impl<I2C, const ADDRESS: u8> Driver8<I2C, ADDRESS> {
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            output_state: 0xff,
        }
    }
}

impl<I2C: crate::I2cBus, const ADDRESS: u8> crate::PortDriver for Driver8<I2C, ADDRESS> {
    type Error = I2C::BusError;

    fn set_high(&mut self, mask: u32) -> Result<(), Self::Error> {
        self.output_state |= mask as u8;
        self.i2c
            .write_reg(ADDRESS, Regs8::OutputPort, self.output_state)
    }
    fn set_low(&mut self, mask: u32) -> Result<(), Self::Error> {
        self.output_state &= !mask as u8;
        self.i2c
            .write_reg(ADDRESS, Regs8::OutputPort, self.output_state)
    }
    fn is_set_high(&mut self, mask: u32) -> Result<bool, Self::Error> {
        Ok(self.output_state & mask as u8 != 0)
    }
    fn is_set_low(&mut self, mask: u32) -> Result<bool, Self::Error> {
        Ok(self.output_state & mask as u8 == 0)
    }

    fn is_high(&mut self, mask: u32) -> Result<bool, Self::Error> {
        Ok(self.i2c.read_reg(ADDRESS, Regs8::InputPort)? & mask as u8 != 0)
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
            .update_reg(ADDRESS, Regs8::Configuration, mask_set, mask_clear)
    }
}
