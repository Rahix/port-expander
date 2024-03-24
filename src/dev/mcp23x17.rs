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
use crate::I2cExt;

/// `MCP23x17` "16-Bit I/O Expander with Serial Interface" with I2C or SPI interface
pub struct Mcp23x17<M>(M);

impl<I2C> Mcp23x17<core::cell::RefCell<Driver<Mcp23017Bus<I2C>>>>
where
    I2C: crate::I2cBus,
{
    /// Create a new instance of the MCP23017 with I2C interface
    pub fn new_mcp23017(bus: I2C, a0: bool, a1: bool, a2: bool) -> Self {
        Self::with_mutex(Mcp23017Bus(bus), a0, a1, a2)
    }
}

impl<SPI> Mcp23x17<core::cell::RefCell<Driver<Mcp23S17Bus<SPI>>>>
where
    SPI: crate::SpiBus,
{
    /// Create a new instance of the MCP23S17 with SPI interface
    pub fn new_mcp23s17(bus: SPI, a0: bool, a1: bool, a2: bool) -> Self {
        Self::with_mutex(Mcp23S17Bus(bus), a0, a1, a2)
    }
}

impl<B, M> Mcp23x17<M>
where
    B: Mcp23x17Bus,
    M: crate::PortMutex<Port = Driver<B>>,
{
    pub fn with_mutex(bus: B, a0: bool, a1: bool, a2: bool) -> Self {
        Self(crate::PortMutex::create(Driver::new(bus, a0, a1, a2)))
    }

    pub fn split<'a>(&'a mut self) -> Parts<'a, B, M> {
        Parts {
            gpa0: crate::Pin::new(0, &self.0),
            gpa1: crate::Pin::new(1, &self.0),
            gpa2: crate::Pin::new(2, &self.0),
            gpa3: crate::Pin::new(3, &self.0),
            gpa4: crate::Pin::new(4, &self.0),
            gpa5: crate::Pin::new(5, &self.0),
            gpa6: crate::Pin::new(6, &self.0),
            gpa7: crate::Pin::new(7, &self.0),
            gpb0: crate::Pin::new(8, &self.0),
            gpb1: crate::Pin::new(9, &self.0),
            gpb2: crate::Pin::new(10, &self.0),
            gpb3: crate::Pin::new(11, &self.0),
            gpb4: crate::Pin::new(12, &self.0),
            gpb5: crate::Pin::new(13, &self.0),
            gpb6: crate::Pin::new(14, &self.0),
            gpb7: crate::Pin::new(15, &self.0),
        }
    }
}

pub struct Parts<'a, B, M = core::cell::RefCell<Driver<B>>>
where
    B: Mcp23x17Bus,
    M: crate::PortMutex<Port = Driver<B>>,
{
    pub gpa0: crate::Pin<'a, crate::mode::Input, M>,
    pub gpa1: crate::Pin<'a, crate::mode::Input, M>,
    pub gpa2: crate::Pin<'a, crate::mode::Input, M>,
    pub gpa3: crate::Pin<'a, crate::mode::Input, M>,
    pub gpa4: crate::Pin<'a, crate::mode::Input, M>,
    pub gpa5: crate::Pin<'a, crate::mode::Input, M>,
    pub gpa6: crate::Pin<'a, crate::mode::Input, M>,
    pub gpa7: crate::Pin<'a, crate::mode::Input, M>,
    pub gpb0: crate::Pin<'a, crate::mode::Input, M>,
    pub gpb1: crate::Pin<'a, crate::mode::Input, M>,
    pub gpb2: crate::Pin<'a, crate::mode::Input, M>,
    pub gpb3: crate::Pin<'a, crate::mode::Input, M>,
    pub gpb4: crate::Pin<'a, crate::mode::Input, M>,
    pub gpb5: crate::Pin<'a, crate::mode::Input, M>,
    pub gpb6: crate::Pin<'a, crate::mode::Input, M>,
    pub gpb7: crate::Pin<'a, crate::mode::Input, M>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// N.B.: These values are for BANK=0, which is the reset state of
/// the chip (and this driver does not change).
///
/// For all registers, the reset value is 0x00, except for
/// IODIR{A,B} which are 0xFF (making all pins inputs) at reset.
enum Regs {
    /// IODIR: input/output direction: 0=output; 1=input
    IODIRA = 0x00,
    /// IPOL: input polarity: 0=register values match input pins; 1=opposite
    IPOLA = 0x02,
    /// GPINTEN: interrupt-on-change: 0=disable; 1=enable
    GPINTENA = 0x04,
    /// DEFVAL: default values for interrupt-on-change
    DEFVALA = 0x06,
    /// INTCON: interrupt-on-change config: 0=compare to previous pin value;
    ///   1=compare to corresponding bit in DEFVAL
    INTCONA = 0x08,
    /// IOCON: configuration register
    /// - Pin 7: BANK (which driver assumes stays 0)
    /// - Pin 6: MIRROR: if enabled, INTA is logically ORed; an interrupt on either
    ///          port will cause both pins to activate
    /// - Pin 5: SEQOP: controls the incrementing function of the address pointer
    /// - Pin 4: DISSLW: disables slew rate control on SDA
    /// - Pin 3: HAEN: no effect on MCP23017
    /// - Pin 2: ODR: interrupt pins are 0=active-driver outputs (INTPOL sets polarity)
    ///          or 1=open-drain outputs (overrides INTPOL)
    /// - Pin 1: INTPOL: interrupt pin is 0=active-low or 1=active-high
    /// - Pin 0: unused
    IOCONA = 0x0a,
    /// GPPU: GPIO pull-ups: enables weak internal pull-ups on each pin (when configured
    ///   as an input)
    GPPUA = 0x0c,
    /// INTF: interrupt flags: 0=no interrupt pending; 1=corresponding pin caused interrupt
    INTFA = 0x0e,
    /// INTCAP: interrupt captured value: reflects value of each pin at the time that they
    ///   caused an interrupt
    INTCAPA = 0x10,
    /// GPIO: reflects logic level on pins
    GPIOA = 0x12,
    /// OLAT: output latches: sets state for pins configured as outputs
    OLATA = 0x14,
    /// IODIR: input/output direction: 0=output; 1=input
    IODIRB = 0x01,
    /// IPOL: input polarity: 0=register values match input pins; 1=opposite
    IPOLB = 0x03,
    /// GPINTEN: interrupt-on-change: 0=disable; 1=enable
    GPINTENB = 0x05,
    /// DEFVAL: default values for interrupt-on-change
    DEFVALB = 0x07,
    /// INTCON: interrupt-on-change config: 0=compare to previous pin value;
    ///   1=compare to corresponding bit in DEFVAL
    INTCONB = 0x09,
    /// IOCON: configuration register
    /// - Pin 7: BANK (which driver assumes stays 0)
    /// - Pin 6: MIRROR: if enabled, INTB is logically ORed; an interrupt on either
    ///          port will cause both pins to activate
    /// - Pin 5: SEQOP: controls the incrementing function of the address pointer
    /// - Pin 4: DISSLW: disables slew rate control on SDA
    /// - Pin 3: HAEN: no effect on MCP23017
    /// - Pin 2: ODR: interrupt pins are 0=active-driver outputs (INTPOL sets polarity)
    ///          or 1=open-drain outputs (overrides INTPOL)
    /// - Pin 1: INTPOL: interrupt pin is 0=active-low or 1=active-high
    /// - Pin 0: unused    INTCONB = 0x09,
    IOCONB = 0x0b,
    /// GPPU: GPIO pull-ups: enables weak internal pull-ups on each pin (when configured
    ///   as an input)
    GPPUB = 0x0d,
    /// INTF: interrupt flags: 0=no interrupt pending; 1=corresponding pin caused interrupt
    INTFB = 0x0f,
    /// INTCAP: interrupt captured value: reflects value of each pin at the time that they
    ///   caused an interrupt
    INTCAPB = 0x11,
    /// GPIO: reflects logic level on pins
    GPIOB = 0x13,
    /// OLAT: output latches: sets state for pins configured as outputs
    OLATB = 0x15,
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
    pub fn new(bus: B, a0: bool, a1: bool, a2: bool) -> Self {
        let addr = 0x20 | ((a2 as u8) << 2) | ((a1 as u8) << 1) | (a0 as u8);
        Self {
            bus,
            out: 0x0000,
            addr,
        }
    }
}

impl<B: Mcp23x17Bus> crate::PortDriver for Driver<B> {
    type Error = B::BusError;

    fn set(&mut self, mask_high: u32, mask_low: u32) -> Result<(), Self::Error> {
        self.out |= mask_high as u16;
        self.out &= !mask_low as u16;
        if (mask_high | mask_low) & 0x00FF != 0 {
            self.bus
                .write_reg(self.addr, Regs::GPIOA, (self.out & 0xFF) as u8)?;
        }
        if (mask_high | mask_low) & 0xFF00 != 0 {
            self.bus
                .write_reg(self.addr, Regs::GPIOB, (self.out >> 8) as u8)?;
        }
        Ok(())
    }

    fn is_set(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error> {
        Ok(((self.out as u32) & mask_high) | (!(self.out as u32) & mask_low))
    }

    fn get(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error> {
        let io0 = if (mask_high | mask_low) & 0x00FF != 0 {
            self.bus.read_reg(self.addr, Regs::GPIOA)?
        } else {
            0
        };
        let io1 = if (mask_high | mask_low) & 0xFF00 != 0 {
            self.bus.read_reg(self.addr, Regs::GPIOB)?
        } else {
            0
        };
        let in_ = ((io1 as u32) << 8) | io0 as u32;
        Ok((in_ & mask_high) | (!in_ & mask_low))
    }
}

impl<B: Mcp23x17Bus> crate::PortDriverTotemPole for Driver<B> {
    fn set_direction(
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
                Regs::IODIRA,
                (mask_set & 0xFF) as u8,
                (mask_clear & 0xFF) as u8,
            )?;
        }
        if mask & 0xFF00 != 0 {
            self.bus.update_reg(
                self.addr,
                Regs::IODIRB,
                (mask_set >> 8) as u8,
                (mask_clear >> 8) as u8,
            )?;
        }
        Ok(())
    }
}

// We need these newtype wrappers since we can't implement `Mcp23x17Bus` for both `I2cBus` and `SpiBus`
// at the same time
pub struct Mcp23017Bus<I2C>(I2C);
pub struct Mcp23S17Bus<SPI>(SPI);

/// Special -Bus trait for the Mcp23x17 since the SPI version is a bit special/weird in terms of writing
/// SPI registers, which can't necessarily be generialized for other devices.
pub trait Mcp23x17Bus {
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

impl<SPI: crate::SpiBus> Mcp23x17Bus for Mcp23S17Bus<SPI> {
    type BusError = SPI::BusError;

    fn write_reg<R: Into<u8>>(
        &mut self,
        addr: u8,
        reg: R,
        value: u8,
    ) -> Result<(), Self::BusError> {
        self.0.write(&[0x40 | addr << 1, reg.into(), value])?;

        Ok(())
    }

    fn read_reg<R: Into<u8>>(&mut self, addr: u8, reg: R) -> Result<u8, Self::BusError> {
        let mut val = [0; 1];
        let write = [0x40 | addr << 1 | 0x1, reg.into()];
        let mut tx = [
            embedded_hal::spi::Operation::Write(&write),
            embedded_hal::spi::Operation::Read(&mut val),
        ];
        self.0.transaction(&mut tx)?;

        Ok(val[0])
    }
}

impl<I2C: crate::I2cBus> Mcp23x17Bus for Mcp23017Bus<I2C> {
    type BusError = I2C::BusError;

    fn write_reg<R: Into<u8>>(
        &mut self,
        addr: u8,
        reg: R,
        value: u8,
    ) -> Result<(), Self::BusError> {
        self.0.write_reg(addr, reg, value)
    }

    fn read_reg<R: Into<u8>>(&mut self, addr: u8, reg: R) -> Result<u8, Self::BusError> {
        self.0.read_reg(addr, reg)
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
            mock_i2c::Transaction::write(0x22, vec![0x12, 0xff]),
            mock_i2c::Transaction::write(0x22, vec![0x12, 0xfe]),
            mock_i2c::Transaction::write(0x22, vec![0x13, 0xff]),
            mock_i2c::Transaction::write(0x22, vec![0x13, 0xfe]),
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
            mock_spi::Transaction::write_vec(vec![0x45, 0x00]),
            mock_spi::Transaction::read(0xff),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x44, 0x00, 0xfe]),
            mock_spi::Transaction::transaction_end(),
            // pin setup gpa7
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x45, 0x00]),
            mock_spi::Transaction::read(0xfe),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x44, 0x00, 0x7e]),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x45, 0x00]),
            mock_spi::Transaction::read(0x7e),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x44, 0x00, 0xfe]),
            mock_spi::Transaction::transaction_end(),
            // pin setup gpb0
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x45, 0x01]),
            mock_spi::Transaction::read(0xff),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x44, 0x01, 0xfe]),
            mock_spi::Transaction::transaction_end(), // pin setup gpb7
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x45, 0x01]),
            mock_spi::Transaction::read(0xfe),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x44, 0x01, 0x7e]),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x45, 0x01]),
            mock_spi::Transaction::read(0x7e),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x44, 0x01, 0xfe]),
            mock_spi::Transaction::transaction_end(),
            // output gpa0, gpb0
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x44, 0x12, 0xff]),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x44, 0x12, 0xfe]),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x44, 0x13, 0xff]),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x44, 0x13, 0xfe]),
            mock_spi::Transaction::transaction_end(),
            // input gpa7, gpb7
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x45, 0x12]),
            mock_spi::Transaction::read(0x80),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x45, 0x12]),
            mock_spi::Transaction::read(0x7f),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x45, 0x13]),
            mock_spi::Transaction::read(0x80),
            mock_spi::Transaction::transaction_end(),
            mock_spi::Transaction::transaction_start(),
            mock_spi::Transaction::write_vec(vec![0x45, 0x13]),
            mock_spi::Transaction::read(0x7f),
            mock_spi::Transaction::transaction_end(),
        ];
        let mut bus = mock_spi::Mock::new(&expectations);

        let mut pca = super::Mcp23x17::new_mcp23s17(bus.clone(), false, true, false);
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
