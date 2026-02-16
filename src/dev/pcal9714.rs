//! Support for the `MCP23017` and `MCP23S17` "16-Bit I/O Expander with Serial Interface"
//!
//! Datasheet: https://ww1.microchip.com/downloads/en/devicedoc/20001952c.pdf
//!
//! The MCP23x17 offers two eight-bit GPIO ports.  It has three
//! address pins, so eight devices can coexist on an I2C bus.
//!
//! Each port has an interrupt, which can be configured to work
//! together or independently.
//!
//! When passing 16-bit values to this driver, the upper byte corresponds to port
//! B (pins 7..0) and the lower byte corresponds to port A (pins 7..0).
use crate::{dev::pca9702::Pca9702Bus, I2cExt};

/// `MCP23x17` "16-Bit I/O Expander with Serial Interface" with I2C or SPI interface
pub struct PCAL9714<M>(M);

// impl<I2C> Mcp23x17<core::cell::RefCell<Driver<Mcp23017Bus<I2C>>>>
// where
//     I2C: crate::I2cBus,
// {
//     /// Create a new instance of the MCP23017 with I2C interface
//     pub fn new_mcp23017(bus: I2C, a0: bool, a1: bool, a2: bool) -> Self {
//         Self::with_mutex(Mcp23017Bus(bus), a0, a1, a2)
//     }
// }

impl<SPI> PCAL9714<core::cell::RefCell<Driver<PCAL9714_Bus<SPI>>>>
where
    SPI: crate::SpiBus,
{
    /// Create a new instance of the MCP23S17 with SPI interface
    pub fn new_PCAL9714(bus: SPI) -> Self {
        Self::with_mutex(PCAL9714_Bus(bus), false, false, false)
    }
}

impl<B, M> PCAL9714<M>
where
    B: PCAL9714Bus,
    M: crate::PortMutex<Port = Driver<B>>,
{
    pub fn with_mutex(bus: B, a0: bool, a1: bool, a2: bool) -> Self {
        Self(crate::PortMutex::create(Driver::new(bus, a0, a1, a2)))
    }

    pub fn split<'a>(&'a mut self) -> Parts<'a, B, M> {
        Parts {
            gp0_0: crate::Pin::new(0, &self.0),
            gp0_1: crate::Pin::new(1, &self.0),
            gp0_2: crate::Pin::new(2, &self.0),
            gp0_3: crate::Pin::new(3, &self.0),
            gp0_4: crate::Pin::new(4, &self.0),
            gp0_5: crate::Pin::new(5, &self.0),
            gp0_6: crate::Pin::new(6, &self.0),
            gp0_7: crate::Pin::new(7, &self.0),
            gp1_0: crate::Pin::new(8, &self.0),
            gp1_1: crate::Pin::new(9, &self.0),
            gp1_2: crate::Pin::new(10, &self.0),
            gp1_3: crate::Pin::new(11, &self.0),
            gp1_4: crate::Pin::new(12, &self.0),
            gp1_5: crate::Pin::new(13, &self.0),
        }
    }
}

pub struct Parts<'a, B, M = core::cell::RefCell<Driver<B>>>
where
    B: PCAL9714Bus,
    M: crate::PortMutex<Port = Driver<B>>,
{
    pub gp0_0: crate::Pin<'a, crate::mode::Input, M>,
    pub gp0_1: crate::Pin<'a, crate::mode::Input, M>,
    pub gp0_2: crate::Pin<'a, crate::mode::Input, M>,
    pub gp0_3: crate::Pin<'a, crate::mode::Input, M>,
    pub gp0_4: crate::Pin<'a, crate::mode::Input, M>,
    pub gp0_5: crate::Pin<'a, crate::mode::Input, M>,
    pub gp0_6: crate::Pin<'a, crate::mode::Input, M>,
    pub gp0_7: crate::Pin<'a, crate::mode::Input, M>,

    pub gp1_0: crate::Pin<'a, crate::mode::Input, M>,
    pub gp1_1: crate::Pin<'a, crate::mode::Input, M>,
    pub gp1_2: crate::Pin<'a, crate::mode::Input, M>,
    pub gp1_3: crate::Pin<'a, crate::mode::Input, M>,
    pub gp1_4: crate::Pin<'a, crate::mode::Input, M>,
    pub gp1_5: crate::Pin<'a, crate::mode::Input, M>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// N.B.: These values are for BANK=0, which is the reset state of
/// the chip (and this driver does not change).
///
/// For all registers, the reset value is 0x00, except for
/// IODIR{A,B} which are 0xFF (making all pins inputs) at reset.
enum Regs {
    /// I/O Direction Register
    InputPort0 = 0x00,
    InputPort1 = 0x01,
    OutputPort0 = 0x02,
    OutputPort1 = 0x03,
    PolarityInversionPort0 = 0x04,
    PolarityInversionPort1 = 0x05,
    ConfigurationPort0 = 0x06,
    ConfigurationPort1 = 0x07,
    OutpurtDriveStrengthRegister0A = 0x40,
    OutpirtDriveStrengthRegister0B = 0x41,
    OutportDriveStrengthRegister1A = 0x42,
    OutportDriveStrengthRegister1B = 0x43,
    InputLatchRegister0 = 0x44,
    InputLatchRegister1 = 0x45,
    PullUpPullDownEnableRegister0 = 0x46,
    PullUpPullDownEnableRegister1 = 0x47,
    PullUpPullDownSelectionRegister0 = 0x48,
    PullUpPullDownSelectionRegister1 = 0x49,
    InteruptMaskRegister0 = 0x4A,
    InteruptMaskRegister1 = 0x4B,
    InteruptStatusRegister0 = 0x4C,
    InteruptStatusRegister1 = 0x4D,
    OutputPortConfigurationRegister = 0x4F,
    InterruptEdgeRegister0A = 0x50,
    InterruptEdgeRegister0B = 0x51,
    InterruptEdgeRegister1A = 0x52,
    InterruptEdgeRegister1B = 0x53,
    InterruptClearRegister0 = 0x54,
    InterruptClearRegister1 = 0x55,
    InputPortReadWithoutInterruptClear0 = 0x56,
    InputPortReadWithoutInterruptClear1 = 0x57,
    OutputConfigurationRegister0 = 0x58,
    OutputConfigurationRegister1 = 0x59,
    SwitchDeboundeEnable0 = 0x5A,
    SwitchDeboundeEnable01 = 0x5B,
    SwitchDebounceCount = 0x5C,
}

impl From<Regs> for u8 {
    fn from(r: Regs) -> u8 {
        r as u8
    }
}

pub struct Driver<B> {
    bus: B,
    out: u16,
    addr: u8,
}

impl<B> Driver<B> {
    pub fn new(bus: B, connected_to_vdd: bool, a1: bool, a2: bool) -> Self {
        // TODO: Add my address here
        // let addr = 0x20 | ((a2 as u8) << 2) | ((a1 as u8) << 1) | (a0 as u8);
        let addr = 0x40 | (connected_to_vdd as u8);
        Self {
            bus,
            out: 0x0000,
            addr,
        }
    }
}

impl<B: PCAL9714Bus> crate::PortDriver for Driver<B> {
    type Error = B::BusError;

    /// Sets the pin high
    fn set(&mut self, mask_high: u32, mask_low: u32) -> Result<(), Self::Error> {
        // TODO: Implement my own set functions, Look at how one of the other implementations does
        // it for the different pin banks. This method may work too.
        self.out |= mask_high as u16;
        self.out &= !mask_low as u16;
        if (mask_high | mask_low) & 0x00FF != 0 {
            self.bus
                .write_reg(self.addr, Regs::OutputPort0, (self.out & 0xFF) as u8)?;
        }
        if (mask_high | mask_low) & 0xFF00 != 0 {
            self.bus
                .write_reg(self.addr, Regs::OutputPort1, (self.out >> 8) as u8)?;
        }
        Ok(())
    }

    fn is_set(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error> {
        Ok(((self.out as u32) & mask_high) | (!(self.out as u32) & mask_low))
    }

    /// Reads the inputpins
    fn get(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error> {
        let io0 = if (mask_high | mask_low) & 0x00FF != 0 {
            self.bus.read_reg(self.addr, Regs::InputPort0)?
        } else {
            0
        };
        let io1 = if (mask_high | mask_low) & 0xFF00 != 0 {
            self.bus.read_reg(self.addr, Regs::InputPort1)?
        } else {
            0
        };
        let in_ = ((io1 as u32) << 8) | io0 as u32;
        Ok((in_ & mask_high) | (!in_ & mask_low))
    }
}

/// This function sets the direction of the pin, if it is an output or an input
impl<B: PCAL9714Bus> crate::PortDriverTotemPole for Driver<B> {
    fn set_direction(
        // TODO: Modify to meet the PCAL9714
        &mut self,
        mask: u32,
        dir: crate::Direction,
        _state: bool,
    ) -> Result<(), Self::Error> {
        let (mask_set, mask_clear) = match dir {
            crate::Direction::Input => (mask as u16, 0),
            crate::Direction::Output => (0, mask as u16),
        };
        if mask & 0x00FF != 0 {
            self.bus.update_reg(
                self.addr,
                Regs::ConfigurationPort0,
                (mask_set & 0xFF) as u8,
                (mask_clear & 0xFF) as u8,
            )?;
        }
        if mask & 0xFF00 != 0 {
            self.bus.update_reg(
                self.addr,
                Regs::ConfigurationPort1,
                (mask_set >> 8) as u8,
                (mask_clear >> 8) as u8,
            )?;
        }
        Ok(())
    }
}

impl<B: PCAL9714Bus> crate::PortDriverPullUp for Driver<B> {
    fn set_pull_up(&mut self, mask: u32, enable: bool) -> Result<(), Self::Error> {
        // TODO: Modify to meet the PCAL9714
        let (mask_set, mask_clear) = match enable {
            true => (mask as u16, 0),
            false => (0, mask as u16),
        };
        if mask & 0x00FF != 0 {
            // Enable/Disable pullup
            self.bus.update_reg(
                self.addr,
                Regs::PullUpPullDownEnableRegister0,
                (mask_set & 0xFF) as u8,
                (mask_clear & 0xFF) as u8,
            )?;
            // Sett 1 then pullup
            self.bus.update_reg(
                self.addr,
                Regs::PullUpPullDownSelectionRegister0,
                (mask_set & 0xFF) as u8,
                (mask_clear & 0xFF) as u8,
            )?;
        }
        if mask & 0xFF00 != 0 {
            // Enable/Disable pullup
            self.bus.update_reg(
                self.addr,
                Regs::PullUpPullDownEnableRegister1,
                (mask_set >> 8) as u8,
                (mask_clear >> 8) as u8,
            )?;
            // Sett 1 then pullup
            self.bus.update_reg(
                self.addr,
                Regs::PullUpPullDownSelectionRegister1,
                (mask_set >> 8) as u8,
                (mask_clear >> 8) as u8,
            )?;
        }
        Ok(())
    }
}

// TODO: write port driver for pull down

impl<B: PCAL9714Bus> crate::PortDriverPolarity for Driver<B> {
    fn set_polarity(&mut self, mask: u32, inverted: bool) -> Result<(), Self::Error> {
        let (mask_set, mask_clear) = match inverted {
            true => (mask as u16, 0),
            false => (0, mask as u16),
        };
        if mask & 0x00FF != 0 {
            self.bus.update_reg(
                self.addr,
                Regs::PolarityInversionPort0,
                (mask_set & 0xFF) as u8,
                (mask_clear & 0xFF) as u8,
            )?;
        }
        if mask & 0xFF00 != 0 {
            self.bus.update_reg(
                self.addr,
                Regs::PolarityInversionPort1,
                (mask_set >> 8) as u8,
                (mask_clear >> 8) as u8,
            )?;
        }
        Ok(())
    }
}

// We need these newtype wrappers since we can't implement `Mcp23x17Bus` for both `I2cBus` and `SpiBus`
// at the same time
// pub struct PCAL9714Bus<I2C>(I2C);
pub struct PCAL9714_Bus<SPI>(SPI);

/// Special -Bus trait for the Mcp23x17 since the SPI version is a bit special/weird in terms of writing
/// SPI registers, which can't necessarily be generialized for other devices.
pub trait PCAL9714Bus {
    type BusError;

    fn write_reg<R: Into<u8>>(&mut self, addr: u8, reg: R, value: u8)
        -> Result<(), Self::BusError>;
    fn read_reg<R: Into<u8>>(&mut self, addr: u8, reg: R) -> Result<u8, Self::BusError>;

    fn update_reg<R: Into<u8>>(
        &mut self,
        addr: u8,
        reg: R,
        mask_set: u8,
        mask_clear: u8,
    ) -> Result<(), Self::BusError> {
        let reg = reg.into();
        let mut val = self.read_reg(addr, reg)?;
        val |= mask_set;
        val &= !mask_clear;
        self.write_reg(addr, reg, val)?;
        Ok(())
    }
}

impl<SPI: crate::SpiBus> PCAL9714Bus for PCAL9714_Bus<SPI> {
    type BusError = SPI::BusError;

    fn write_reg<R: Into<u8>>(
        &mut self,
        addr: u8,
        reg: R,
        value: u8,
    ) -> Result<(), Self::BusError> {
        let conf = [
            // (32 << 1) & !0x01,
            addr & !0x01,
            reg.into(),
            value, // All outputs
        ];

        self.0.write(&conf)?;

        Ok(())
    }

    fn read_reg<R: Into<u8>>(&mut self, addr: u8, reg: R) -> Result<u8, Self::BusError> {
        let mut val = [0; 1];
        let write = [0x40 | addr << 1 | 0x1, reg.into()];
        let mut tx = [
            // TODO: Modify to meet the correct embedded hal things
            embedded_hal::spi::Operation::Write(&write),
            embedded_hal::spi::Operation::Read(&mut val),
        ];
        self.0.transaction(&mut tx)?;

        Ok(val[0])
    }
}

#[cfg(test)]
mod tests {
    use embedded_hal_mock::eh1::{i2c as mock_i2c, spi as mock_spi};

    #[test]
    fn mcp23017() {
        let expectations = [
            // pin setup gpa0
            mock_i2c::Transaction::write_read(0x22, vec![0x00], vec![0xff]),
            mock_i2c::Transaction::write(0x22, vec![0x00, 0xfe]),
            // pin setup gpa7
            mock_i2c::Transaction::write_read(0x22, vec![0x00], vec![0xfe]),
            mock_i2c::Transaction::write(0x22, vec![0x00, 0x7e]),
            mock_i2c::Transaction::write_read(0x22, vec![0x00], vec![0x7e]),
            mock_i2c::Transaction::write(0x22, vec![0x00, 0xfe]),
            // pin setup gpb0
            mock_i2c::Transaction::write_read(0x22, vec![0x01], vec![0xff]),
            mock_i2c::Transaction::write(0x22, vec![0x01, 0xfe]),
            // pin setup gpb7
            mock_i2c::Transaction::write_read(0x22, vec![0x01], vec![0xfe]),
            mock_i2c::Transaction::write(0x22, vec![0x01, 0x7e]),
            mock_i2c::Transaction::write_read(0x22, vec![0x01], vec![0x7e]),
            mock_i2c::Transaction::write(0x22, vec![0x01, 0xfe]),
            // output gpa0, gpb0
            mock_i2c::Transaction::write(0x22, vec![0x12, 0x01]),
            mock_i2c::Transaction::write(0x22, vec![0x12, 0x00]),
            mock_i2c::Transaction::write(0x22, vec![0x13, 0x01]),
            mock_i2c::Transaction::write(0x22, vec![0x13, 0x00]),
            // input gpa7, gpb7
            mock_i2c::Transaction::write_read(0x22, vec![0x12], vec![0x80]),
            mock_i2c::Transaction::write_read(0x22, vec![0x12], vec![0x7f]),
            mock_i2c::Transaction::write_read(0x22, vec![0x13], vec![0x80]),
            mock_i2c::Transaction::write_read(0x22, vec![0x13], vec![0x7f]),
        ];
        let mut bus = mock_i2c::Mock::new(&expectations);

        let mut pca = super::Mcp23x17::new_mcp23017(bus.clone(), false, true, false);
        let pca_pins = pca.split();

        let mut gpa0 = pca_pins.gpa0.into_output().unwrap();
        let gpa7 = pca_pins.gpa7.into_output().unwrap();
        let gpa7 = gpa7.into_input().unwrap();

        let mut gpb0 = pca_pins.gpb0.into_output().unwrap();
        let gpb7 = pca_pins.gpb7.into_output().unwrap();
        let gpb7 = gpb7.into_input().unwrap();

        // output high and low
        gpa0.set_high().unwrap();
        gpa0.set_low().unwrap();
        gpb0.set_high().unwrap();
        gpb0.set_low().unwrap();

        // input high and low
        assert!(gpa7.is_high().unwrap());
        assert!(gpa7.is_low().unwrap());
        assert!(gpb7.is_high().unwrap());
        assert!(gpb7.is_low().unwrap());

        bus.done();
    }

    #[test]
    fn mcp23s17() {
        let expectations = [
            // pin setup gpa0
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x41, 0x00]),
            mock_spi::Transaction::read(0xff),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x40, 0x00, 0xfe]),
            mock_spi::Transaction::transaction_end(),
            // pin setup gpa7
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x41, 0x00]),
            mock_spi::Transaction::read(0xfe),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x40, 0x00, 0x7e]),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x41, 0x00]),
            mock_spi::Transaction::read(0x7e),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x40, 0x00, 0xfe]),
            mock_spi::Transaction::transaction_end(),
            // pin setup gpb0
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x41, 0x01]),
            mock_spi::Transaction::read(0xff),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x40, 0x01, 0xfe]),
            mock_spi::Transaction::transaction_end(), // pin setup gpb7
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x41, 0x01]),
            mock_spi::Transaction::read(0xfe),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x40, 0x01, 0x7e]),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x41, 0x01]),
            mock_spi::Transaction::read(0x7e),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x40, 0x01, 0xfe]),
            mock_spi::Transaction::transaction_end(),
            // output gpa0, gpb0
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x40, 0x12, 0x01]),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x40, 0x12, 0x00]),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x40, 0x13, 0x01]),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x40, 0x13, 0x00]),
            mock_spi::Transaction::transaction_end(),
            // input gpa7, gpb7
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x41, 0x12]),
            mock_spi::Transaction::read(0x80),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x41, 0x12]),
            mock_spi::Transaction::read(0x7f),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x41, 0x13]),
            mock_spi::Transaction::read(0x80),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x41, 0x13]),
            mock_spi::Transaction::read(0x7f),
            mock_spi::Transaction::transaction_end(),
        ];
        let mut bus = mock_spi::Mock::new(&expectations);

        let mut pca = super::Mcp23x17::new_mcp23s17(bus.clone());
        let pca_pins = pca.split();

        let mut gpa0 = pca_pins.gpa0.into_output().unwrap();
        let gpa7 = pca_pins.gpa7.into_output().unwrap();
        let gpa7 = gpa7.into_input().unwrap();

        let mut gpb0 = pca_pins.gpb0.into_output().unwrap();
        let gpb7 = pca_pins.gpb7.into_output().unwrap();
        let gpb7 = gpb7.into_input().unwrap();

        // output high and low
        gpa0.set_high().unwrap();
        gpa0.set_low().unwrap();
        gpb0.set_high().unwrap();
        gpb0.set_low().unwrap();

        // input high and low
        assert!(gpa7.is_high().unwrap());
        assert!(gpa7.is_low().unwrap());
        assert!(gpb7.is_high().unwrap());
        assert!(gpb7.is_low().unwrap());

        bus.done();
    }
}
