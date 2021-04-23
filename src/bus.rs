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
