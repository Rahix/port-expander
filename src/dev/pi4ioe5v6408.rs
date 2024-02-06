//! Support for the `PI4IOE5V6408` "Low-voltage Translating 8-bit I2C-bus I/O Expander"
use crate::I2cExt;

/// `PI4IOE5V6408` "Low-voltage Translating 8-bit I2C-bus I/O Expander"
pub struct Pi4ioe5v6408<M>(M);

impl<I2C> Pi4ioe5v6408<shared_bus::NullMutex<Driver<I2C>>>
where
    I2C: crate::I2cBus,
{
    /// Create a new driver for the `PI4IOE5V6408` "Low-voltage Translating 8-bit I2C-bus I/O Expander"
    /// All pins will be configured as floating inputs
    ///
    /// # Arguments
    /// - `i2c` - The I2C bus the device is connected to
    /// - `addr` - The address of the device. The address is 0x43 if `addr` is `false` and 0x44 if `addr` is `true`
    pub fn new(i2c: I2C, addr: bool) -> Result<Self, I2C::BusError> {
        Self::with_mutex(i2c, addr)
    }
}

impl<I2C, M> Pi4ioe5v6408<M>
where
    I2C: crate::I2cBus,
    M: shared_bus::BusMutex<Bus = Driver<I2C>>,
{
    /// Create a new driver for the `PI4IOE5V6408` "Low-voltage Translating 8-bit I2C-bus I/O Expander"
    /// with a mutex.
    /// All pins will be configured as floating inputs
    ///
    /// # Arguments
    /// - `i2c` - The I2C bus the device is connected to
    /// - `addr` - The address of the device. The address is 0x43 if `addr` is `false` and 0x44 if `addr` is `true`
    pub fn with_mutex(i2c: I2C, addr: bool) -> Result<Self, I2C::BusError> {
        Ok(Self(shared_bus::BusMutex::create(Driver::new(
            i2c, addr, false,
        )?)))
    }

    /// Create a new driver for the `PI4IOE5V6408` "Low-voltage Translating 8-bit I2C-bus I/O Expander"
    /// retaining the previous (pullup/down and interrupt) configuration.
    ///
    /// Warning: Only use this constructor to recreate the driver for a chip that has been properly initialized before.
    ///
    /// # Arguments
    /// - `i2c` - The I2C bus the device is connected to
    /// - `addr` - The address of the device. The address is 0x43 if `addr` is `false` and 0x44 if `addr` is `true`
    pub fn with_retained_pin_config(i2c: I2C, addr: bool) -> Result<Self, I2C::BusError> {
        Ok(Self(shared_bus::BusMutex::create(Driver::new(
            i2c, addr, true,
        )?)))
    }

    pub fn split<'a>(&'a mut self) -> Parts<'a, I2C, M> {
        Parts {
            io0: crate::Pin::new(0, &self.0),
            io1: crate::Pin::new(1, &self.0),
            io2: crate::Pin::new(2, &self.0),
            io3: crate::Pin::new(3, &self.0),
            io4: crate::Pin::new(4, &self.0),
            io5: crate::Pin::new(5, &self.0),
            io6: crate::Pin::new(6, &self.0),
            io7: crate::Pin::new(7, &self.0),
        }
    }
}

pub struct Parts<'a, I2C, M = shared_bus::NullMutex<Driver<I2C>>>
where
    I2C: crate::I2cBus,
    M: shared_bus::BusMutex<Bus = Driver<I2C>>,
{
    pub io0: crate::Pin<'a, crate::mode::Input, M>,
    pub io1: crate::Pin<'a, crate::mode::Input, M>,
    pub io2: crate::Pin<'a, crate::mode::Input, M>,
    pub io3: crate::Pin<'a, crate::mode::Input, M>,
    pub io4: crate::Pin<'a, crate::mode::Input, M>,
    pub io5: crate::Pin<'a, crate::mode::Input, M>,
    pub io6: crate::Pin<'a, crate::mode::Input, M>,
    pub io7: crate::Pin<'a, crate::mode::Input, M>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Regs {
    DeviceIdControl = 0x01,
    IODirection = 0x03,
    OutputPort = 0x05,
    OutputHighImpedance = 0x07,
    InputDefaultState = 0x09,
    PullUpPullDownEnable = 0x0b,
    PullUpPullDownSelection = 0x0d,
    InputStatusRegister = 0x0f,
    InterruptMaskRegister = 0x11,
    InterruptStatusRegister = 0x13,
}

impl From<Regs> for u8 {
    fn from(r: Regs) -> u8 {
        r as u8
    }
}

pub struct Driver<I2C> {
    i2c: I2C,
    addr: u8,
    out: u8,
}

impl<I2C: crate::I2cBus> Driver<I2C> {
    pub fn new(mut i2c: I2C, addr: bool, retain_config: bool) -> Result<Self, I2C::BusError> {
        let addr = if addr { 0x44 } else { 0x43 };

        let device_id = i2c.read_reg(addr, Regs::DeviceIdControl)?; // Reset the "(Power on) Reset Interrupt" bit (and validate the device ID)
        assert_eq!(
            device_id & 0xFC, // Only check Manufacturer ID (0b101) and Firmware Revision (0b000)
            0xA0,
            "Unexpected Device ID for the PI4IOE5V6408: 0x{:02x}",
            device_id
        );

        // The Reset values are the following:

        // i2c.write_reg(addr, Regs::IODirection, 0)?; // All pins as inputs
        // i2c.write_reg(addr, Regs::OutputPort, 0)?; // Set all outputs to low
        // i2c.write_reg(addr, Regs::OutputHighImpedance, 0xff)?; // Set high impedance mode on all outputs
        // i2c.write_reg(addr, Regs::InputDefaultState, 0)?; // The default state of all inputs is 0
        // i2c.write_reg(addr, Regs::PullUpPullDownEnable, 0xff)?; // Pull-Up/Pull-Down enabled on all inputs
        // i2c.write_reg(addr, Regs::PullUpPullDownSelection, 0)?; // Pull-Downs on all inputs
        // i2c.write_reg(addr, Regs::InterruptMaskRegister, 0)?; // Interrupts enabled on all inputs

        let mut out = 0;

        if retain_config {
            out = i2c.read_reg(addr, Regs::OutputPort)?; // Read the current output state once
        } else {
            // First time this driver is initialized, after it has been reset: Change reset values we don't want
            i2c.write_reg(addr, Regs::OutputHighImpedance, 0)?; // Disable high impedance mode on all outputs
            i2c.write_reg(addr, Regs::InterruptMaskRegister, 0xff)?; // Disable interrupts on all inputs
            i2c.write_reg(addr, Regs::PullUpPullDownEnable, 0)?; // Disable pull-up/pull-down on all inputs
        }

        Ok(Self { i2c, addr, out })
    }
}

impl<I2C: crate::I2cBus> crate::PortDriver for Driver<I2C> {
    type Error = I2C::BusError;

    fn set(&mut self, mask_high: u32, mask_low: u32) -> Result<(), Self::Error> {
        let previous = self.out;
        self.out |= mask_high as u8;
        self.out &= !mask_low as u8;
        if self.out != previous {
            self.i2c.write_reg(self.addr, Regs::OutputPort, self.out)
        } else {
            // don't do the transfer when nothing changed
            Ok(())
        }
    }

    fn is_set(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error> {
        Ok(((self.out as u32) & mask_high) | (!(self.out as u32) & mask_low))
    }

    fn get(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error> {
        let in_ = self.i2c.read_reg(self.addr, Regs::InputStatusRegister)? as u32;
        Ok((in_ & mask_high) | (!in_ & mask_low))
    }
}

impl<I2C: crate::I2cBus> crate::PortDriverTotemPole for Driver<I2C> {
    fn set_direction(
        &mut self,
        mask: u32,
        dir: crate::Direction,
        state: bool,
    ) -> Result<(), Self::Error> {
        // set state before switching direction to prevent glitch
        if dir == crate::Direction::Output {
            use crate::PortDriver;
            if state {
                self.set(mask, 0)?;
            } else {
                self.set(0, mask)?;
            }
        }

        let (mask_set, mask_clear) = match dir {
            crate::Direction::Output => (mask as u8, 0), // Outputs are set to 1
            crate::Direction::Input => (0, mask as u8),  // Inputs are set to 0
        };
        self.i2c
            .update_reg(self.addr, Regs::IODirection, mask_set, mask_clear)
    }
}

#[cfg(test)]
mod tests {
    use embedded_hal_mock::eh1::i2c as mock_i2c;
    use shared_bus::NullMutex;

    #[test]
    fn pi4ioe5v6408() {
        let expectations = [
            // driver setup
            mock_i2c::Transaction::write_read(0x43, vec![0x01], vec![0xa2]),
            mock_i2c::Transaction::write(0x43, vec![0x07, 0b00000000]),
            mock_i2c::Transaction::write(0x43, vec![0x11, 0b11111111]),
            mock_i2c::Transaction::write(0x43, vec![0x0b, 0b00000000]),
            // pin setup io0
            mock_i2c::Transaction::write_read(0x43, vec![0x03], vec![0]),
            mock_i2c::Transaction::write(0x43, vec![0x03, 0b00000001]),
            // pin setup io1
            mock_i2c::Transaction::write(0x43, vec![0x05, 0b00000010]),
            mock_i2c::Transaction::write_read(0x43, vec![0x03], vec![0b00000001]),
            mock_i2c::Transaction::write(0x43, vec![0x03, 0b00000011]),
            // pin setup io0 as input
            mock_i2c::Transaction::write_read(0x43, vec![0x03], vec![0b00000011]),
            mock_i2c::Transaction::write(0x43, vec![0x03, 0b00000010]),
            // io1 writes
            mock_i2c::Transaction::write(0x43, vec![0x05, 0b00000000]),
            mock_i2c::Transaction::write(0x43, vec![0x05, 0b00000010]),
            mock_i2c::Transaction::write(0x43, vec![0x05, 0b00000000]),
            // io0 reads
            mock_i2c::Transaction::write_read(0x43, vec![0x0f], vec![0b00000001]),
            mock_i2c::Transaction::write_read(0x43, vec![0x0f], vec![0b00000000]),
        ];
        let mut bus = mock_i2c::Mock::new(&expectations);

        let mut pca = super::Pi4ioe5v6408::new(bus.clone(), false).unwrap();
        let pca_pins = pca.split();

        let io0 = pca_pins.io0.into_output().unwrap();
        let mut io1 = pca_pins.io1.into_output_high().unwrap();

        let io0 = io0.into_input().unwrap();

        io1.set_low().unwrap();
        io1.set_high().unwrap();
        io1.toggle().unwrap();

        assert!(io0.is_high().unwrap());
        assert!(io0.is_low().unwrap());

        bus.done();
    }

    #[test]
    fn pi4ioe5v6408_retained() {
        let expectations = [
            // driver setup
            mock_i2c::Transaction::write_read(0x44, vec![0x01], vec![0xa2]),
            mock_i2c::Transaction::write_read(0x44, vec![0x05], vec![0b10101111]),
            // pin setup io0
            mock_i2c::Transaction::write(0x44, vec![0x05, 0b10101110]),
            mock_i2c::Transaction::write_read(0x44, vec![0x03], vec![0]),
            mock_i2c::Transaction::write(0x44, vec![0x03, 0b00000001]),
            // pin setup io1
            mock_i2c::Transaction::write_read(0x44, vec![0x03], vec![0b00000001]),
            mock_i2c::Transaction::write(0x44, vec![0x03, 0b00000011]),
            // io1 writes
            mock_i2c::Transaction::write(0x44, vec![0x05, 0b10101100]),
            mock_i2c::Transaction::write(0x44, vec![0x05, 0b10101110]),
            mock_i2c::Transaction::write(0x44, vec![0x05, 0b10101100]),
        ];
        let mut bus = mock_i2c::Mock::new(&expectations);

        let mut pca: super::Pi4ioe5v6408<NullMutex<_>> =
            super::Pi4ioe5v6408::with_retained_pin_config(bus.clone(), true).unwrap();
        let pca_pins = pca.split();

        let _io0 = pca_pins.io0.into_output().unwrap();
        let mut io1 = pca_pins.io1.into_output_high().unwrap();

        io1.set_low().unwrap();
        io1.set_high().unwrap();
        io1.toggle().unwrap();

        bus.done();
    }
}
