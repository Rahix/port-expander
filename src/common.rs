use core::marker::PhantomData;
use embedded_hal::blocking::i2c as hal_i2c;
use embedded_hal::digital::v2 as hal_digital;

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

pub struct Pin<'a, MODE, MUTEX, PORT>
where
    PORT: Port,
    MUTEX: shared_bus::BusMutex<Bus = PORT::Driver>,
{
    pin_mask: u32,
    port_driver: &'a MUTEX,
    _p: PhantomData<MODE>,
    _m: PhantomData<PORT>,
}

impl<'a, MODE, MUTEX, PORT> Pin<'a, MODE, MUTEX, PORT>
where
    PORT: Port,
    MUTEX: shared_bus::BusMutex<Bus = PORT::Driver>,
{
    pub fn new(pin_number: u8, port_driver: &'a MUTEX) -> Self {
        assert!(pin_number < 32);
        Self {
            pin_mask: 1 << pin_number,
            port_driver,
            _p: PhantomData,
            _m: PhantomData,
        }
    }

    pub fn into_input(
        self,
    ) -> Result<Pin<'a, mode::Input, MUTEX, PORT>, <PORT::Driver as PortDriver>::Error> {
        self.port_driver
            .lock(|drv| drv.set_direction(self.pin_mask, Direction::Input))?;
        Ok(Pin {
            pin_mask: self.pin_mask,
            port_driver: self.port_driver,
            _p: PhantomData,
            _m: PhantomData,
        })
    }

    pub fn into_output(
        self,
    ) -> Result<Pin<'a, mode::Output, MUTEX, PORT>, <PORT::Driver as PortDriver>::Error> {
        self.port_driver
            .lock(|drv| drv.set_direction(self.pin_mask, Direction::Output))?;
        Ok(Pin {
            pin_mask: self.pin_mask,
            port_driver: self.port_driver,
            _p: PhantomData,
            _m: PhantomData,
        })
    }
}

impl<'a, MUTEX, PORT> Pin<'a, mode::Input, MUTEX, PORT>
where
    PORT: Port,
    MUTEX: shared_bus::BusMutex<Bus = PORT::Driver>,
{
    pub fn is_high(&self) -> Result<bool, <PORT::Driver as PortDriver>::Error> {
        self.port_driver.lock(|drv| drv.is_high(self.pin_mask))
    }

    pub fn is_low(&self) -> Result<bool, <PORT::Driver as PortDriver>::Error> {
        self.port_driver.lock(|drv| drv.is_low(self.pin_mask))
    }
}

impl<'a, MUTEX, PORT> hal_digital::InputPin for Pin<'a, mode::Input, MUTEX, PORT>
where
    PORT: Port,
    MUTEX: shared_bus::BusMutex<Bus = PORT::Driver>,
{
    type Error = <PORT::Driver as PortDriver>::Error;

    fn is_high(&self) -> Result<bool, Self::Error> {
        Pin::is_high(self)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Pin::is_low(self)
    }
}

impl<'a, MUTEX, PORT> Pin<'a, mode::Output, MUTEX, PORT>
where
    PORT: Port,
    MUTEX: shared_bus::BusMutex<Bus = PORT::Driver>,
{
    pub fn set_high(&self) -> Result<(), <PORT::Driver as PortDriver>::Error> {
        self.port_driver.lock(|drv| drv.set_high(self.pin_mask))
    }

    pub fn set_low(&self) -> Result<(), <PORT::Driver as PortDriver>::Error> {
        self.port_driver.lock(|drv| drv.set_low(self.pin_mask))
    }

    pub fn is_set_high(&self) -> Result<bool, <PORT::Driver as PortDriver>::Error> {
        self.port_driver.lock(|drv| drv.is_set_high(self.pin_mask))
    }

    pub fn is_set_low(&self) -> Result<bool, <PORT::Driver as PortDriver>::Error> {
        self.port_driver.lock(|drv| drv.is_set_low(self.pin_mask))
    }

    pub fn toggle(&self) -> Result<(), <PORT::Driver as PortDriver>::Error> {
        self.port_driver.lock(|drv| drv.toggle(self.pin_mask))
    }
}

impl<'a, MUTEX, PORT> hal_digital::OutputPin for Pin<'a, mode::Output, MUTEX, PORT>
where
    PORT: Port,
    MUTEX: shared_bus::BusMutex<Bus = PORT::Driver>,
{
    type Error = <PORT::Driver as PortDriver>::Error;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        Pin::set_low(self)
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Pin::set_high(self)
    }
}

impl<'a, MUTEX, PORT> hal_digital::StatefulOutputPin for Pin<'a, mode::Output, MUTEX, PORT>
where
    PORT: Port,
    MUTEX: shared_bus::BusMutex<Bus = PORT::Driver>,
{
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Pin::is_set_high(self)
    }

    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Pin::is_set_low(self)
    }
}

impl<'a, MUTEX, PORT> hal_digital::ToggleableOutputPin for Pin<'a, mode::Output, MUTEX, PORT>
where
    PORT: Port,
    MUTEX: shared_bus::BusMutex<Bus = PORT::Driver>,
{
    type Error = <PORT::Driver as PortDriver>::Error;

    fn toggle(&mut self) -> Result<(), Self::Error> {
        Pin::toggle(self)
    }
}
