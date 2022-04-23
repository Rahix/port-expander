//! Support for the `MCP23017` "16-Bit I/O Expander with Serial Interface"
//!
//! Datasheet: https://ww1.microchip.com/downloads/en/devicedoc/20001952c.pdf
//!
//! The MCP23017 offers two eight-bit GPIO ports.  It has three
//! address pins, so eight devices can coexist on an I2C bus.
//!
//! Each port has an interrupt, which can be configured to work
//! together or independently.
//!
//! When passing 16-bit values to this driver, the upper byte corresponds to port
//! B (pins 7..0) and the lower byte corresponds to port B (pins 7..0).
use crate::I2cExt;

/// `MCP23017` "16-Bit I/O Expander with Serial Interface"
pub struct Mcp23017<M>(M);

impl<I2C> Mcp23017<shared_bus::NullMutex<Driver<I2C>>>
where
    I2C: crate::I2cBus,
{
    pub fn new(i2c: I2C, a0: bool, a1: bool, a2: bool) -> Self {
        Self::with_mutex(i2c, a0, a1, a2)
    }
}

impl<I2C, M> Mcp23017<M>
where
    I2C: crate::I2cBus,
    M: shared_bus::BusMutex<Bus = Driver<I2C>>,
{
    pub fn with_mutex(i2c: I2C, a0: bool, a1: bool, a2: bool) -> Self {
        Self(shared_bus::BusMutex::create(Driver::new(i2c, a0, a1, a2)))
    }

    pub fn split<'a>(&'a mut self) -> Parts<'a, I2C, M> {
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

pub struct Parts<'a, I2C, M = shared_bus::NullMutex<Driver<I2C>>>
where
    I2C: crate::I2cBus,
    M: shared_bus::BusMutex<Bus = Driver<I2C>>,
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
enum Regs {
    // N.B.: These values are for BANK=0, which is the reset state of
    // the chip (and this driver does not change).
    //
    // For all registers, the reset value is 0x00, except for
    // IODIR{A,B} which are 0xFF (making all pins inputs) at reset.
    //
    // IODIR: input/output direction: 0=output; 1=input
    // IPOL: input polarity: 0=register values match input pins; 1=opposite
    // GPINTEN: interrupt-on-change: 0=disable; 1=enable
    // DEFVAL: default values for interrupt-on-change
    // INTCON: interrupt-on-change config: 0=compare to previous pin value;
    //   1=compare to corresponding bit in DEFVAL
    // IOCON: configuration register
    // - Pin 7: BANK (which driver assumes stays 0)
    // - Pin 6: MIRROR: if enabled, INT{A,B} are logically ORed; an interrupt on either
    //          port will cause both pins to activate
    // - Pin 5: SEQOP: controls the incrementing function of the address pointer
    // - Pin 4: DISSLW: disables slew rate control on SDA
    // - Pin 3: HAEN: no effect on MCP23017
    // - Pin 2: ODR: interrupt pins are 0=active-driver outputs (INTPOL sets polarity)
    //          or 1=open-drain outputs (overrides INTPOL)
    // - Pin 1: INTPOL: interrupt pin is 0=active-low or 1=active-high
    // - Pin 0: unused
    // GPPU: GPIO pull-ups: enables weak internal pull-ups on each pin (when configured
    //   as an input)
    // INTF: interrupt flags: 0=no interrupt pending; 1=corresponding pin caused interrupt
    // INTCAP: interrupt captured value: reflects value of each pin at the time that they
    //   caused an interrupt
    // GPIO: reflects logic level on pins
    // OLAT: output latches: sets state for pins configured as outputs
    IODIRA = 0x00,
    IPOLA = 0x02,
    GPINTENA = 0x04,
    DEFVALA = 0x06,
    INTCONA = 0x08,
    IOCONA = 0x0a,
    GPPUA = 0x0c,
    INTFA = 0x0e,
    INTCAPA = 0x10,
    GPIOA = 0x12,
    OLATA = 0x14,
    IODIRB = 0x01,
    IPOLB = 0x03,
    GPINTENB = 0x05,
    DEFVALB = 0x07,
    INTCONB = 0x09,
    IOCONB = 0x0b,
    GPPUB = 0x0d,
    INTFB = 0x0f,
    INTCAPB = 0x11,
    GPIOB = 0x13,
    OLATB = 0x15,
}

impl From<Regs> for u8 {
    fn from(r: Regs) -> u8 {
        r as u8
    }
}

pub struct Driver<I2C> {
    i2c: I2C,
    out: u16,
    addr: u8,
}

impl<I2C> Driver<I2C> {
    pub fn new(i2c: I2C, a0: bool, a1: bool, a2: bool) -> Self {
        let addr = 0x20 | ((a2 as u8) << 2) | ((a1 as u8) << 1) | (a0 as u8);
        Self {
            i2c,
            out: 0xffff,
            addr,
        }
    }
}

impl<I2C: crate::I2cBus> crate::PortDriver for Driver<I2C> {
    type Error = I2C::BusError;

    fn set(&mut self, mask_high: u32, mask_low: u32) -> Result<(), Self::Error> {
        self.out |= mask_high as u16;
        self.out &= !mask_low as u16;
        if (mask_high | mask_low) & 0x00FF != 0 {
            self.i2c
                .write_reg(self.addr, Regs::GPIOA, (self.out & 0xFF) as u8)?;
        }
        if (mask_high | mask_low) & 0xFF00 != 0 {
            self.i2c
                .write_reg(self.addr, Regs::GPIOB, (self.out >> 8) as u8)?;
        }
        Ok(())
    }

    fn is_set(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error> {
        Ok(((self.out as u32) & mask_high) | (!(self.out as u32) & mask_low))
    }

    fn get(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error> {
        let io0 = if (mask_high | mask_low) & 0x00FF != 0 {
            self.i2c.read_reg(self.addr, Regs::GPIOA)?
        } else {
            0
        };
        let io1 = if (mask_high | mask_low) & 0xFF00 != 0 {
            self.i2c.read_reg(self.addr, Regs::GPIOB)?
        } else {
            0
        };
        let in_ = ((io1 as u32) << 8) | io0 as u32;
        Ok((in_ & mask_high) | (!in_ & mask_low))
    }
}

impl<I2C: crate::I2cBus> crate::PortDriverTotemPole for Driver<I2C> {
    fn set_direction(&mut self, mask: u32, dir: crate::Direction) -> Result<(), Self::Error> {
        let (mask_set, mask_clear) = match dir {
            crate::Direction::Input => (mask as u16, 0),
            crate::Direction::Output => (0, mask as u16),
        };
        if mask & 0x00FF != 0 {
            self.i2c.update_reg(
                self.addr,
                Regs::IODIRA,
                (mask_set & 0xFF) as u8,
                (mask_clear & 0xFF) as u8,
            )?;
        }
        if mask & 0xFF00 != 0 {
            self.i2c.update_reg(
                self.addr,
                Regs::IODIRB,
                (mask_set >> 8) as u8,
                (mask_clear >> 8) as u8,
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use embedded_hal_mock::i2c as mock_i2c;

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

        let mut pca = super::Mcp23017::new(bus.clone(), false, true, false);
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
