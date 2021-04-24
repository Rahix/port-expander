use embedded_hal::blocking::i2c as hal_i2c;

/// Blanket trait for types implementing `i2c::WriteRead + i2c::Write`
pub trait I2cBus: hal_i2c::WriteRead + hal_i2c::Write + hal_i2c::Read {
    type BusError: From<<Self as hal_i2c::WriteRead>::Error>
        + From<<Self as hal_i2c::Write>::Error>
        + From<<Self as hal_i2c::Read>::Error>;
}

impl<T, E> I2cBus for T
where
    T: hal_i2c::WriteRead<Error = E> + hal_i2c::Write<Error = E> + hal_i2c::Read<Error = E>,
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
