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
