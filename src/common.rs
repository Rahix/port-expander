use embedded_hal::blocking::i2c as hal_i2c;

/// Blanket trait for types implementing `i2c::WriteRead + i2c::Write`
pub trait I2cBus: hal_i2c::WriteRead + hal_i2c::Write {
    type BusError: From<<Self as hal_i2c::WriteRead>::Error> + From<<Self as hal_i2c::Write>::Error>;
}

impl<T, E> I2cBus for T
where
    T: hal_i2c::WriteRead<Error = E> + hal_i2c::Write<Error = E>,
{
    type BusError = E;
}

pub trait Port {
    type Driver: PortDriver;
}

pub trait PortDriver {
    type Error;

    fn set_high(&mut self, mask: u32) -> Result<(), Self::Error>;
    fn set_low(&mut self, mask: u32) -> Result<(), Self::Error>;
    fn is_set_high(&mut self, mask: u32) -> Result<bool, Self::Error>;
    fn is_set_low(&mut self, mask: u32) -> Result<bool, Self::Error>;

    fn is_high(&mut self, mask: u32) -> Result<bool, Self::Error>;
    fn is_low(&mut self, mask: u32) -> Result<bool, Self::Error>;

    fn set_direction(&mut self, mask: u32, dir: Direction) -> Result<(), Self::Error>;

    fn toggle(&mut self, mask: u32) -> Result<(), Self::Error> {
        if self.is_set_high(mask)? {
            self.set_low(mask)?;
        } else {
            self.set_high(mask)?;
        }
        Ok(())
    }
}

pub enum Direction {
    Input,
    Output,
}

pub mod mode {
    pub struct Input;
    pub struct Output;
}
