use embedded_hal::{i2c as hal_i2c, spi as hal_spi};

/// Blanket trait for types implementing `i2c::I2c
pub trait I2cBus: hal_i2c::I2c {
    type BusError: From<<Self as hal_i2c::ErrorType>::Error>;
}

impl<T, E> I2cBus for T
where
    T: hal_i2c::I2c<Error = E>,
{
    type BusError = E;
}

pub(crate) trait I2cExt {
    type Error;

    fn write_reg<R: Into<u8>>(&mut self, addr: u8, reg: R, value: u8) -> Result<(), Self::Error>;
    fn update_reg<R: Into<u8>>(
        &mut self,
        addr: u8,
        reg: R,
        mask_set: u8,
        mask_clear: u8,
    ) -> Result<(), Self::Error>;
    fn read_reg<R: Into<u8>>(&mut self, addr: u8, reg: R) -> Result<u8, Self::Error>;
}

impl<I2C: I2cBus> I2cExt for I2C {
    type Error = I2C::BusError;

    fn write_reg<R: Into<u8>>(&mut self, addr: u8, reg: R, value: u8) -> Result<(), Self::Error> {
        self.write(addr, &[reg.into(), value])?;
        Ok(())
    }

    fn update_reg<R: Into<u8>>(
        &mut self,
        addr: u8,
        reg: R,
        mask_set: u8,
        mask_clear: u8,
    ) -> Result<(), Self::Error> {
        let reg = reg.into();
        let mut buf = [0x00];
        self.write_read(addr, &[reg], &mut buf)?;
        buf[0] |= mask_set;
        buf[0] &= !mask_clear;
        self.write(addr, &[reg, buf[0]])?;
        Ok(())
    }

    fn read_reg<R: Into<u8>>(&mut self, addr: u8, reg: R) -> Result<u8, Self::Error> {
        let mut buf = [0x00];
        self.write_read(addr, &[reg.into()], &mut buf)?;
        Ok(buf[0])
    }
}

pub trait SpiBus: hal_spi::SpiDevice {
    type BusError: From<<Self as hal_spi::ErrorType>::Error>;
}

impl<T, E> SpiBus for T
where
    T: hal_spi::SpiDevice<Error = E>,
{
    type BusError = E;
}
