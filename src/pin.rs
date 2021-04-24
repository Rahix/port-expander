use core::marker::PhantomData;
use embedded_hal::digital::v2 as hal_digital;

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
    MUTEX: shared_bus::BusMutex<Bus = PD>,
{
    pub(crate) fn new(pin_number: u8, port_driver: &'a MUTEX) -> Self {
        assert!(pin_number < 32);
        Self {
            pin_mask: 1 << pin_number,
            port_driver,
            _m: PhantomData,
        }
    }
}

impl<'a, MODE, MUTEX, PD> Pin<'a, MODE, MUTEX>
where
    PD: crate::PortDriver + crate::PortDriverTotemPole,
    MUTEX: shared_bus::BusMutex<Bus = PD>,
{
    pub fn into_input(self) -> Result<Pin<'a, crate::mode::Input, MUTEX>, PD::Error> {
        self.port_driver
            .lock(|drv| drv.set_direction(self.pin_mask, crate::Direction::Input))?;
        Ok(Pin {
            pin_mask: self.pin_mask,
            port_driver: self.port_driver,
            _m: PhantomData,
        })
    }

    pub fn into_output(self) -> Result<Pin<'a, crate::mode::Output, MUTEX>, PD::Error> {
        self.port_driver
            .lock(|drv| drv.set_direction(self.pin_mask, crate::Direction::Output))?;
        Ok(Pin {
            pin_mask: self.pin_mask,
            port_driver: self.port_driver,
            _m: PhantomData,
        })
    }
}

impl<'a, MODE: crate::mode::HasInput, MUTEX, PD> Pin<'a, MODE, MUTEX>
where
    PD: crate::PortDriver,
    MUTEX: shared_bus::BusMutex<Bus = PD>,
{
    pub fn is_high(&self) -> Result<bool, PD::Error> {
        self.port_driver.lock(|drv| drv.is_high(self.pin_mask))
    }

    pub fn is_low(&self) -> Result<bool, PD::Error> {
        self.port_driver.lock(|drv| drv.is_low(self.pin_mask))
    }
}

impl<'a, MODE: crate::mode::HasInput, MUTEX, PD> hal_digital::InputPin for Pin<'a, MODE, MUTEX>
where
    PD: crate::PortDriver,
    MUTEX: shared_bus::BusMutex<Bus = PD>,
{
    type Error = PD::Error;

    fn is_high(&self) -> Result<bool, Self::Error> {
        Pin::is_high(self)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Pin::is_low(self)
    }
}

impl<'a, MODE: crate::mode::HasOutput, MUTEX, PD> Pin<'a, MODE, MUTEX>
where
    PD: crate::PortDriver,
    MUTEX: shared_bus::BusMutex<Bus = PD>,
{
    pub fn set_high(&mut self) -> Result<(), PD::Error> {
        self.port_driver.lock(|drv| drv.set_high(self.pin_mask))
    }

    pub fn set_low(&mut self) -> Result<(), PD::Error> {
        self.port_driver.lock(|drv| drv.set_low(self.pin_mask))
    }

    pub fn is_set_high(&self) -> Result<bool, PD::Error> {
        self.port_driver.lock(|drv| drv.is_set_high(self.pin_mask))
    }

    pub fn is_set_low(&self) -> Result<bool, PD::Error> {
        self.port_driver.lock(|drv| drv.is_set_low(self.pin_mask))
    }

    pub fn toggle(&mut self) -> Result<(), PD::Error> {
        self.port_driver.lock(|drv| drv.toggle(self.pin_mask))
    }
}

impl<'a, MODE: crate::mode::HasOutput, MUTEX, PD> hal_digital::OutputPin for Pin<'a, MODE, MUTEX>
where
    PD: crate::PortDriver,
    MUTEX: shared_bus::BusMutex<Bus = PD>,
{
    type Error = PD::Error;

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
    PD: crate::PortDriver,
    MUTEX: shared_bus::BusMutex<Bus = PD>,
{
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Pin::is_set_high(self)
    }

    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Pin::is_set_low(self)
    }
}

impl<'a, MODE: crate::mode::HasOutput, MUTEX, PD> hal_digital::ToggleableOutputPin
    for Pin<'a, MODE, MUTEX>
where
    PD: crate::PortDriver,
    MUTEX: shared_bus::BusMutex<Bus = PD>,
{
    type Error = PD::Error;

    fn toggle(&mut self) -> Result<(), Self::Error> {
        Pin::toggle(self)
    }
}
