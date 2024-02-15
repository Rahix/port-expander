use core::marker::PhantomData;
use embedded_hal::digital::{self as hal_digital, ErrorType};

/// Representation of a port-expander pin.
///
/// `Pin` is not constructed directly, this type is created by instanciating a port-expander and
/// then getting access to all its pins using the `.split()` method.
pub struct Pin<'a, MODE, MUTEX> {
    pin_mask: u32,
    port_driver: &'a MUTEX,
    _m: PhantomData<MODE>,
}

impl<'a, MODE, MUTEX, PD> Pin<'a, MODE, MUTEX>
where
    PD: crate::PortDriver,
    MUTEX: crate::PortMutex<Port = PD>,
{
    pub(crate) fn new(pin_number: u8, port_driver: &'a MUTEX) -> Self {
        assert!(pin_number < 32);
        Self {
            pin_mask: 1 << pin_number,
            port_driver,
            _m: PhantomData,
        }
    }

    pub fn pin_mask(&self) -> u32 {
        self.pin_mask
    }

    pub(crate) fn port_driver(&self) -> &MUTEX {
        self.port_driver
    }

    pub fn access_port_driver<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut PD) -> R,
    {
        self.port_driver.lock(|pd| f(pd))
    }
}
impl<'a, MODE, MUTEX, PD> ErrorType for Pin<'a, MODE, MUTEX>
where
    PD: crate::PortDriver + crate::PortDriverTotemPole,
    PD::Error: embedded_hal::digital::Error,
    MUTEX: crate::PortMutex<Port = PD>,
{
    type Error = PD::Error;
}
impl<'a, MODE, MUTEX, PD> Pin<'a, MODE, MUTEX>
where
    PD: crate::PortDriver + crate::PortDriverTotemPole,
    MUTEX: crate::PortMutex<Port = PD>,
{
    /// Configure this pin as an input.
    ///
    /// The exact electrical details depend on the port-expander device which is used.
    pub fn into_input(self) -> Result<Pin<'a, crate::mode::Input, MUTEX>, PD::Error> {
        self.port_driver
            .lock(|drv| drv.set_direction(self.pin_mask, crate::Direction::Input, false))?;
        Ok(Pin {
            pin_mask: self.pin_mask,
            port_driver: self.port_driver,
            _m: PhantomData,
        })
    }

    /// Configure this pin as an output with an initial LOW state.
    ///
    /// The LOW state is, as long as he port-expander chip allows this, entered without any
    /// electrical glitch.
    pub fn into_output(self) -> Result<Pin<'a, crate::mode::Output, MUTEX>, PD::Error> {
        self.port_driver
            .lock(|drv| drv.set_direction(self.pin_mask, crate::Direction::Output, false))?;
        Ok(Pin {
            pin_mask: self.pin_mask,
            port_driver: self.port_driver,
            _m: PhantomData,
        })
    }

    /// Configure this pin as an output with an initial HIGH state.
    ///
    /// The HIGH state is, as long as he port-expander chip allows this, entered without any
    /// electrical glitch.
    pub fn into_output_high(self) -> Result<Pin<'a, crate::mode::Output, MUTEX>, PD::Error> {
        self.port_driver
            .lock(|drv| drv.set_direction(self.pin_mask, crate::Direction::Output, true))?;
        Ok(Pin {
            pin_mask: self.pin_mask,
            port_driver: self.port_driver,
            _m: PhantomData,
        })
    }
}

impl<'a, MODE, MUTEX, PD> Pin<'a, MODE, MUTEX>
where
    PD: crate::PortDriver + crate::PortDriverPolarity,
    MUTEX: crate::PortMutex<Port = PD>,
{
    /// Turn on hardware polarity inversion for this pin.
    pub fn into_inverted(self) -> Result<Self, PD::Error> {
        self.port_driver
            .lock(|drv| drv.set_polarity(self.pin_mask, true))?;
        Ok(self)
    }

    /// Set hardware polarity inversion for this pin.
    pub fn set_inverted(&mut self, inverted: bool) -> Result<(), PD::Error> {
        self.port_driver
            .lock(|drv| drv.set_polarity(self.pin_mask, inverted))?;
        Ok(())
    }
}

impl<'a, MODE: crate::mode::HasInput, MUTEX, PD> Pin<'a, MODE, MUTEX>
where
    PD: crate::PortDriver,
    MUTEX: crate::PortMutex<Port = PD>,
{
    /// Read the pin's input state and return `true` if it is HIGH.
    pub fn is_high(&self) -> Result<bool, PD::Error> {
        self.port_driver
            .lock(|drv| Ok(drv.get(self.pin_mask, 0)? == self.pin_mask))
    }

    /// Read the pin's input state and return `true` if it is LOW.
    pub fn is_low(&self) -> Result<bool, PD::Error> {
        self.port_driver
            .lock(|drv| Ok(drv.get(0, self.pin_mask)? == self.pin_mask))
    }
}

impl<'a, MODE: crate::mode::HasInput, MUTEX, PD> hal_digital::InputPin for Pin<'a, MODE, MUTEX>
where
    PD: crate::PortDriver + crate::PortDriverTotemPole,
    <PD as crate::PortDriver>::Error: embedded_hal::digital::Error,
    MUTEX: crate::PortMutex<Port = PD>,
{
    fn is_high(&mut self) -> Result<bool, <PD as crate::PortDriver>::Error> {
        Pin::is_high(self)
    }

    fn is_low(&mut self) -> Result<bool, <PD as crate::PortDriver>::Error> {
        Pin::is_low(self)
    }
}

impl<'a, MODE: crate::mode::HasOutput, MUTEX, PD> Pin<'a, MODE, MUTEX>
where
    PD: crate::PortDriver,
    MUTEX: crate::PortMutex<Port = PD>,
{
    /// Set the pin's output state to HIGH.
    ///
    /// Note that this can have different electrical meanings depending on the port-expander chip.
    pub fn set_high(&mut self) -> Result<(), PD::Error> {
        self.port_driver.lock(|drv| drv.set(self.pin_mask, 0))
    }

    /// Set the pin's output state to LOW.
    ///
    /// Note that this can have different electrical meanings depending on the port-expander chip.
    pub fn set_low(&mut self) -> Result<(), PD::Error> {
        self.port_driver.lock(|drv| drv.set(0, self.pin_mask))
    }

    /// Return `true` if the pin's output state is HIGH.
    ///
    /// This method does **not** read the pin's electrical state.
    pub fn is_set_high(&self) -> Result<bool, PD::Error> {
        self.port_driver
            .lock(|drv| Ok(drv.is_set(self.pin_mask, 0)? == self.pin_mask))
    }

    /// Return `true` if the pin's output state is LOW.
    ///
    /// This method does **not** read the pin's electrical state.
    pub fn is_set_low(&self) -> Result<bool, PD::Error> {
        self.port_driver
            .lock(|drv| Ok(drv.is_set(0, self.pin_mask)? == self.pin_mask))
    }

    /// Toggle the pin's output state.
    pub fn toggle(&mut self) -> Result<(), PD::Error> {
        self.port_driver.lock(|drv| drv.toggle(self.pin_mask))
    }
}

impl<'a, MODE: crate::mode::HasOutput, MUTEX, PD> hal_digital::OutputPin for Pin<'a, MODE, MUTEX>
where
    PD: crate::PortDriver + crate::PortDriverTotemPole,
    <PD as crate::PortDriver>::Error: embedded_hal::digital::Error,
    MUTEX: crate::PortMutex<Port = PD>,
{
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Pin::set_low(self)
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Pin::set_high(self)
    }
}

impl<'a, MODE: crate::mode::HasOutput, MUTEX, PD> hal_digital::StatefulOutputPin
    for Pin<'a, MODE, MUTEX>
where
    PD: crate::PortDriver + crate::PortDriverTotemPole,
    <PD as crate::PortDriver>::Error: embedded_hal::digital::Error,
    MUTEX: crate::PortMutex<Port = PD>,
{
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Pin::is_set_high(self)
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Pin::is_set_low(self)
    }

    fn toggle(&mut self) -> Result<(), Self::Error> {
        Pin::toggle(self)
    }
}
