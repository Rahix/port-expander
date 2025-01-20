use embedded_hal::spi::Operation;
use crate::{SpiBus, PortDriver};

/// Error type for PCA9702 driver.
/// We wrap both bus errors and "unsupported operation" errors here.
#[derive(Debug)]
pub enum Pca9702Error<E> {
    /// Underlying SPI bus error
    Bus(E),
    /// Called an output-only function on an input-only device
    OutputNotSupported,
}

/// An 8-bit input-only expander with SPI, based on the PCA9702.
pub struct Pca9702<M>(M);

impl<SPI> Pca9702<core::cell::RefCell<Driver<Pca9702Bus<SPI>>>> 
where
    SPI: crate::SpiBus,
{
    pub fn new(bus: SPI) -> Self {
        Self::with_mutex(Pca9702Bus(bus))
    }
}

impl<B, M> Pca9702<M>
where
    B: Pca9702BusTrait,
    M: crate::PortMutex<Port = Driver<B>>,
{
    /// Create a PCA9702 driver with a user-supplied mutex type.
    pub fn with_mutex(bus: B) -> Self {
        Self(crate::PortMutex::create(Driver::new(bus)))
    }

    /// Split this device into its 8 input pins.
    ///
    /// All pins are always configured as inputs on PCA9702 hardware.
    pub fn split<'a>(&'a mut self) -> Parts<'a, B, M> {
        Parts {
            in0: crate::Pin::new(0, &self.0),
            in1: crate::Pin::new(1, &self.0),
            in2: crate::Pin::new(2, &self.0),
            in3: crate::Pin::new(3, &self.0),
            in4: crate::Pin::new(4, &self.0),
            in5: crate::Pin::new(5, &self.0),
            in6: crate::Pin::new(6, &self.0),
            in7: crate::Pin::new(7, &self.0),
        }
    }
}

/// Container for all 8 input pins on the PCA9702.
pub struct Parts<'a, B, M = core::cell::RefCell<Driver<B>>>
where
    B: Pca9702BusTrait,
    M: crate::PortMutex<Port = Driver<B>>,
{
    pub in0: crate::Pin<'a, crate::mode::Input, M>,
    pub in1: crate::Pin<'a, crate::mode::Input, M>,
    pub in2: crate::Pin<'a, crate::mode::Input, M>,
    pub in3: crate::Pin<'a, crate::mode::Input, M>,
    pub in4: crate::Pin<'a, crate::mode::Input, M>,
    pub in5: crate::Pin<'a, crate::mode::Input, M>,
    pub in6: crate::Pin<'a, crate::mode::Input, M>,
    pub in7: crate::Pin<'a, crate::mode::Input, M>,
}

/// Internal driver struct for PCA9702.
pub struct Driver<B> {
    bus: B,
}

impl<B> Driver<B> {
    fn new(bus: B) -> Self {
        Self { bus }
    }
}

/// Trait for the underlying PCA9702 SPI bus. Simpler than e.g. MCP23S17
/// because PCA9702 is read-only and has no register-based protocol.
pub trait Pca9702BusTrait {
    type BusError;

    /// Reads 8 bits from the device (which represent the state of inputs [in7..in0])
    fn read_inputs(&mut self) -> Result<u8, Self::BusError>;
}

impl<B: Pca9702BusTrait> PortDriver for Driver<B> {
    /// Our `Error` is a custom enum wrapping both bus errors and an unsupported-ops error.
    type Error = Pca9702Error<B::BusError>;

    /// PCA9702 is input-only, return an error here.
    fn set(&mut self, _mask_high: u32, _mask_low: u32) -> Result<(), Self::Error> {
        Err(Pca9702Error::OutputNotSupported)
    }

    /// PCA9702 is input-only, return an error here.
    fn is_set(&mut self, _mask_high: u32, _mask_low: u32) -> Result<u32, Self::Error> {
        Err(Pca9702Error::OutputNotSupported)
    }

    /// Read the actual input bits from the PCA9702 device
    fn get(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error> {
        let val = self
            .bus
            .read_inputs()
            .map_err(Pca9702Error::Bus)? as u32;
        Ok((val & mask_high) | (!val & mask_low))
    }
}

/// Bus wrapper type for PCA9702, implementing `Pca9702BusTrait`.
pub struct Pca9702Bus<SPI>(pub SPI);

impl<SPI> Pca9702BusTrait for Pca9702Bus<SPI>
where
    SPI: SpiBus,
{
    type BusError = SPI::BusError;

    fn read_inputs(&mut self) -> Result<u8, Self::BusError> {
        // PCA9702 wants a total of 8 SCLK rising edges to shift out the input data
        // from SDOUT: The first rising edge latches the inputs, the next 8 edges
        // shift them out.
        let mut buffer = [0u8];
        let mut ops = [Operation::TransferInPlace(&mut buffer)];
        self.0.transaction(&mut ops)?;

        // buffer[0] now holds bits [in7..in0]
        Ok(buffer[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};

    #[test]
    fn pca9702_read_inputs() {
        let expectations = [
            // 1st read
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![0], vec![0xA5]),
            SpiTransaction::transaction_end(),
    
            // 2nd read
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![0], vec![0xA5]),
            SpiTransaction::transaction_end(),
    
            // 3rd read
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![0], vec![0xA5]),
            SpiTransaction::transaction_end(),
        ];
        let spi_mock = SpiMock::new(&expectations);
        let mut pca = Pca9702::new(spi_mock);
        let pins = pca.split();
    
        // For each call, the driver re-reads from SPI, returning 0xA5 each time.
        // 0xA5 = 0b10100101 => in0=1, in1=0, in2=1, in3=0, in4=0, in5=1, in6=0, in7=1
        assert_eq!(pins.in0.is_high().unwrap(), true);   // LSB => 1
        assert_eq!(pins.in1.is_high().unwrap(), false);
        assert_eq!(pins.in2.is_high().unwrap(), true);
    
        // Finally, consume the mock:
        let mut spi = pca.0.into_inner().bus.0;
        spi.done();
    }

    #[test]
    fn pca9702_output_fails() {
        let spi_mock = SpiMock::new(&[]);
        let pca = Pca9702::new(spi_mock);

        let err = pca.0.borrow_mut().set(0x01, 0).unwrap_err();
        match err {
            Pca9702Error::OutputNotSupported => {},
            _ => panic!("Expected OutputNotSupported error"),
        }
        let mut spi = pca.0.into_inner().bus.0;
        spi.done(); 
    }
}
