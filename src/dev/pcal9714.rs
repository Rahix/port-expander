//! Support for the `PCAL9714` "Ultra low-voltage translating 14-bit SPI I/O expander with Agile I/O features, interrupt output, and reset"
//!
//! Datasheet: https://www.nxp.com/docs/en/data-sheet/PCAL9714.pdf
//!         (Archive: https://web.archive.org/web/20260102051402/https://www.nxp.com/docs/en/data-sheet/PCAL9714.pdf)
//!
//! The PCAL9714 offers one eight-bit GPIO port and one six-bit GPIO port.
//! It has two possible addresses so one chip select can be used for two IC's.
//!
//! Each port has an interrupt, which can be configured to work
//! together or independently.
//!
//! When passing 16-bit values to this driver, the upper byte corresponds to port
//! 1 (pins 5..0) and the lower byte corresponds to port 0 (pins 7..0).

/// `PCAL9714` "14-Bit I/O Expander with Agile I/O features, interrupt output, and reset" with SPI interface
pub struct PCAL9714<M>(M);

impl<SPI> PCAL9714<core::cell::RefCell<Driver<PCAL9714_Bus<SPI>>>>
where
    SPI: crate::SpiBus,
{
    /// Create a new instance of the PCAL9714 with SPI interface
    pub fn new_PCAL9714(bus: SPI, address_pin: bool) -> Self {
        Self::with_mutex(PCAL9714_Bus(bus), address_pin, false, false)
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
    OutportDriveStrengthRegister0A = 0x40,
    OutportDriveStrengthRegister0B = 0x41,
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
    SwitchDebounceEnable0 = 0x5A,
    SwitchDebounceEnable1 = 0x5B,
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
    pub fn new(bus: B, address_pin: bool, _a1: bool, _a2: bool) -> Self {
        // Address pin is connected to V_ss (-> 0x40) or V_dd(-> 0x42)
        let addr = match address_pin {
            true => 0x42,
            false => 0x40,
        };
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
impl<B: PCAL9714Bus> crate::PortDriverPullDown for Driver<B> {
    fn set_pull_down(&mut self, mask: u32, enable: bool) -> Result<(), Self::Error> {
        let (mask_set, mask_clear) = match enable {
            true => (mask as u16, 0),
            false => (0, mask as u16),
        };
        if mask & 0x00FF != 0 {
            // Enable/Disable pullup
            self.bus.update_reg(
                self.addr,
                Regs::PullUpPullDownEnableRegister0,
                !((mask_set & 0xFF) as u8),
                !((mask_clear & 0xFF) as u8),
            )?;
            // Sett 1 then pullup
            self.bus.update_reg(
                self.addr,
                Regs::PullUpPullDownSelectionRegister0,
                !((mask_set & 0xFF) as u8),
                !((mask_clear & 0xFF) as u8),
            )?;
        }
        if mask & 0xFF00 != 0 {
            // Enable/Disable pullup
            self.bus.update_reg(
                self.addr,
                Regs::PullUpPullDownEnableRegister1,
                !((mask_set >> 8) as u8),
                !((mask_clear >> 8) as u8),
            )?;
            // Sett 1 then pullup
            self.bus.update_reg(
                self.addr,
                Regs::PullUpPullDownSelectionRegister1,
                !((mask_set >> 8) as u8),
                !((mask_clear >> 8) as u8),
            )?;
        }
        Ok(())
    }
}

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
        let conf = [addr, reg.into(), value];

        self.0.write(&conf)?;

        Ok(())
    }

    fn read_reg<R: Into<u8>>(&mut self, addr: u8, reg: R) -> Result<u8, Self::BusError> {
        let mut val = [0; 1];
        let addr = addr | 0x01;
        let write = [addr, reg.into()];
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
    use super::*;
    use embedded_hal_mock::eh1::spi as mock_spi;
    use log;
    use pretty_env_logger;

    #[test]
    fn pcal9714() {
        // Init logging
        let _ = pretty_env_logger::formatted_builder()
            .is_test(true)
            .filter(None, log::LevelFilter::Debug)
            .try_init();

        // Making SPI expectation list
        // See datasheet for more specific details for the transactions.
        //
        // Reading the PCAL9714:
        //      Write: [Addr, Register]
        //      Read: [Value]
        //
        // Writing to the PCAL9714:
        //      Write: [Addr, Register, Value]
        //
        //
        // Address:
        //      For reading: 0x41
        //      For Writing: 0x40
        let expectations = [
            // Pin setup gp0_1
            //      Reading the current pin configuration
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x41, Regs::ConfigurationPort0.into()]),
            mock_spi::Transaction::read(0xff),
            mock_spi::Transaction::transaction_end(),
            //      Setting the pin as an output
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x40, Regs::ConfigurationPort0.into(), 0xfe]),
            mock_spi::Transaction::transaction_end(),
            // Pin setup of gp0_7
            mock_spi::Transaction::transaction_start(),
            //      Reading the current pin configuration
            mock_spi::Transaction::write_vec(vec![0x41, Regs::ConfigurationPort0.into()]),
            mock_spi::Transaction::read(0xfe),
            mock_spi::Transaction::transaction_end(),
            //      Setting the pin as an output
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x40, Regs::ConfigurationPort0.into(), 0x7e]),
            mock_spi::Transaction::transaction_end(),
            //      Reading the pin configuration
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x41, Regs::ConfigurationPort0.into()]),
            mock_spi::Transaction::read(0x7e),
            mock_spi::Transaction::transaction_end(),
            //      Setting the pin as an input
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x40, Regs::ConfigurationPort0.into(), 0xfe]),
            mock_spi::Transaction::transaction_end(),
            // Pin setup of gp1_0
            //      Reading current port configuration
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x41, Regs::ConfigurationPort1.into()]),
            mock_spi::Transaction::read(0xff),
            mock_spi::Transaction::transaction_end(),
            //      Setting the pin as an output
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x40, Regs::ConfigurationPort1.into(), 0xfe]),
            mock_spi::Transaction::transaction_end(),
            // Pin setup of gp1_5
            //      Reading current port configuration
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x41, Regs::ConfigurationPort1.into()]),
            mock_spi::Transaction::read(0xfe),
            mock_spi::Transaction::transaction_end(),
            //      Setting pin as an output
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x40, Regs::ConfigurationPort1.into(), 0xDE]),
            mock_spi::Transaction::transaction_end(),
            //      Reading pin configuration
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x41, Regs::ConfigurationPort1.into()]),
            mock_spi::Transaction::read(0xde),
            mock_spi::Transaction::transaction_end(),
            //      Setting pin as input
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x40, Regs::ConfigurationPort1.into(), 0xfe]),
            mock_spi::Transaction::transaction_end(),
            // Setting gp0_0 high
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x40, Regs::OutputPort0.into(), 0x01]),
            mock_spi::Transaction::transaction_end(),
            // Setting gp0_0 low
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x40, Regs::OutputPort0.into(), 0x00]),
            mock_spi::Transaction::transaction_end(),
            // Setting gp1_0 high
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x40, Regs::OutputPort1.into(), 0x01]),
            mock_spi::Transaction::transaction_end(),
            // Setting gp1_0 low
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x40, Regs::OutputPort1.into(), 0x00]),
            mock_spi::Transaction::transaction_end(),
            // input gp0_7, gp1_5
            // Reading the value of gp0_7
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x41, Regs::InputPort0.into()]),
            mock_spi::Transaction::read(0x80), // TODO: check
            mock_spi::Transaction::transaction_end(),
            // Reading the value of gp0_7
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x41, Regs::InputPort0.into()]),
            mock_spi::Transaction::read(0x7f),
            mock_spi::Transaction::transaction_end(),
            // Reading the value of gp1_5
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x41, Regs::InputPort1.into()]),
            mock_spi::Transaction::read(0xB0),
            mock_spi::Transaction::transaction_end(),
            // Reading the value of gp1_5
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x41, Regs::InputPort1.into()]),
            mock_spi::Transaction::read(0xDf),
            mock_spi::Transaction::transaction_end(),
        ];

        println!("INFO: Starting Mock SPI");
        let mut bus = mock_spi::Mock::new(&expectations);

        println!("INFO: Configuring the port expander");
        let mut pca = super::PCAL9714::new_PCAL9714(bus.clone(), false);
        let pca_pins = pca.split();

        println!("INFO: Setting gp0_0 as output");
        let mut gp0_0 = pca_pins.gp0_0.into_output().unwrap();

        println!("INFO: Setting gp0_7 as output");
        let gp0_7 = pca_pins.gp0_7.into_output().unwrap();
        println!("INFO: Setting gp0_7 as input");
        let gp0_7 = gp0_7.into_input().unwrap();

        println!("INFO: Setting gp1_0 as output");
        let mut gp1_0 = pca_pins.gp1_0.into_output().unwrap();
        println!("INFO: Setting gp1_5 as output");
        let gp1_5 = pca_pins.gp1_5.into_output().unwrap();
        println!("INFO: Setting gp1_5 as output");
        let gp1_5 = gp1_5.into_input().unwrap();

        // output high and low
        println!("INFO: Setting gp0_0 high");
        gp0_0.set_high().unwrap();
        println!("INFO: Setting gp0_0 low");
        gp0_0.set_low().unwrap();
        println!("INFO: Setting gp1_0 high");
        gp1_0.set_high().unwrap();
        println!("INFO: Setting gp1_0 low");
        gp1_0.set_low().unwrap();

        println!("INFO: Asserting the inputs");

        // input high and low
        println!("INFO: Asserting gp07 is high?");
        assert!(gp0_7.is_high().unwrap());
        println!("INFO: Asserting gp07 is low?");
        assert!(gp0_7.is_low().unwrap());
        println!("INFO: Asserting gp1_5 is high?");
        assert!(gp1_5.is_high().unwrap());
        println!("INFO: Asserting gp1_5 is low?");
        assert!(gp1_5.is_low().unwrap());

        bus.done();
    }
}
