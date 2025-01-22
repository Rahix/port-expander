//! Support for the `PCA9702` "8-Bit Input-Only Expander with SPI"
//!
//! Datasheet: https://www.nxp.com/docs/en/data-sheet/PCA9701_PCA9702.pdf
//!
//! The PCA9702 offers eight input pins, with an interrupt output that can be asserted when
//! one or more of the inputs change state (enabled via the `INT_EN` pin). The device reads
//! its inputs on each falling edge of `CS` and then presents them on `SDOUT` bit-by-bit as
//! the clock (SCLK) rises.
//!
//! Because the PCA9702 is strictly input-only, there is no way to drive output values or
//! configure directions. Consequently, calling methods that attempt to write or read back
//! “set” states (e.g., `set()`, `is_set()`) will return an error.

use crate::{Pin, PortDriver, SpiBus};
use embedded_hal::spi::Operation;

#[cfg(feature = "async")]
use crate::pin_async::{AsyncPortState, InterruptHandler, PinAsync};
#[cfg(feature = "async")]
use core::cell::RefCell;

/// An 8-bit input-only expander with SPI, based on the PCA9702.
///
/// Internally, this struct is a tuple:
///
/// - `.0` is your mutex-wrapped driver (`PortMutex<Driver<...>>`)
/// - `.1` is an internal `RefCell<AsyncPortState>` used for optional async functionality
///
/// See [`split()`] for synchronous usage, or [`split_async()`] for async usage.
pub struct Pca9702<M>(
    /// The core driver behind a `PortMutex`, e.g. `RefCell<Driver<Pca9702Bus<SPI>>>`.
    pub M,
    /// Internal asynchronous state (used only if you call `split_async()`).
    #[cfg(feature = "async")]
    pub RefCell<AsyncPortState>,
    // If the "async" feature is not enabled, we simply don't store any
    // extra field. For build consistency, let's conditionalize it:
    #[cfg(not(feature = "async"))] (),
);

impl<SPI> Pca9702<core::cell::RefCell<Driver<Pca9702Bus<SPI>>>>
where
    SPI: SpiBus,
{
    /// Create a PCA9702 driver using a `core::cell::RefCell` as the mutex.
    ///
    /// This is the simplest constructor for no-std usage. If you'd prefer a different
    /// mutex type, call the low-level [`with_mutex()`] constructor.
    pub fn new(bus: SPI) -> Self {
        Self::with_mutex(Pca9702Bus(bus))
    }
}

impl<B, M> Pca9702<M>
where
    B: Pca9702BusTrait,
    M: crate::PortMutex<Port = Driver<B>>,
{
    /// Create a PCA9702 driver with a user-supplied mutex type, e.g. a critical-section mutex or a std mutex.
    pub fn with_mutex(bus: B) -> Self {
        #[cfg(feature = "async")]
        {
            Self(
                crate::PortMutex::create(Driver::new(bus)),
                RefCell::new(AsyncPortState::new()),
            )
        }

        #[cfg(not(feature = "async"))]
        {
            // If async is disabled, there's no second field
            Self(crate::PortMutex::create(Driver::new(bus)), ())
        }
    }

    /// Split this device into its 8 input pins (synchronous usage).
    ///
    /// All pins are always configured as inputs on PCA9702 hardware.
    pub fn split(&mut self) -> Parts<'_, B, M> {
        Parts {
            in0: Pin::new(0, &self.0),
            in1: Pin::new(1, &self.0),
            in2: Pin::new(2, &self.0),
            in3: Pin::new(3, &self.0),
            in4: Pin::new(4, &self.0),
            in5: Pin::new(5, &self.0),
            in6: Pin::new(6, &self.0),
            in7: Pin::new(7, &self.0),
        }
    }

    /// **Async** split: returns 8 async input pins plus an interrupt handler.
    ///
    /// 1. Reads the device once to sync `AsyncPortState` with the real pin values.
    /// 2. Returns [`PartsAsync`] with `PinAsync` plus an [`InterruptHandler`].
    ///
    /// You must call `.handle_interrupts()` from your hardware ISR to wake any tasks
    /// that are waiting on pin changes.
    #[cfg(feature = "async")]
    pub fn split_async<'a>(&'a mut self) -> Result<PartsAsync<'a, B, M>, B::BusError> {
        // Perform an initial read so the async state doesn't see a spurious edge
        let initial_state = self.0.lock(|drv| drv.get(0xFF, 0))?;
        self.1.borrow_mut().last_known_state = initial_state;

        Ok(PartsAsync {
            in0: PinAsync::new(Pin::new(0, &self.0), &self.1, 0),
            in1: PinAsync::new(Pin::new(1, &self.0), &self.1, 1),
            in2: PinAsync::new(Pin::new(2, &self.0), &self.1, 2),
            in3: PinAsync::new(Pin::new(3, &self.0), &self.1, 3),
            in4: PinAsync::new(Pin::new(4, &self.0), &self.1, 4),
            in5: PinAsync::new(Pin::new(5, &self.0), &self.1, 5),
            in6: PinAsync::new(Pin::new(6, &self.0), &self.1, 6),
            in7: PinAsync::new(Pin::new(7, &self.0), &self.1, 7),

            interrupts: InterruptHandler::new(&self.0, &self.1),
        })
    }
}

/// Container for all 8 input pins on the PCA9702 (synchronous).
pub struct Parts<'a, B, M = core::cell::RefCell<Driver<B>>>
where
    B: Pca9702BusTrait,
    M: crate::PortMutex<Port = Driver<B>>,
{
    pub in0: Pin<'a, crate::mode::Input, M>,
    pub in1: Pin<'a, crate::mode::Input, M>,
    pub in2: Pin<'a, crate::mode::Input, M>,
    pub in3: Pin<'a, crate::mode::Input, M>,
    pub in4: Pin<'a, crate::mode::Input, M>,
    pub in5: Pin<'a, crate::mode::Input, M>,
    pub in6: Pin<'a, crate::mode::Input, M>,
    pub in7: Pin<'a, crate::mode::Input, M>,
}

#[cfg(feature = "async")]
/// The async version of `Parts`, with `PinAsync`s plus an [`InterruptHandler`].
pub struct PartsAsync<'a, B, M>
where
    B: Pca9702BusTrait,
    M: crate::PortMutex<Port = Driver<B>>,
{
    pub in0: PinAsync<'a, crate::mode::Input, M>,
    pub in1: PinAsync<'a, crate::mode::Input, M>,
    pub in2: PinAsync<'a, crate::mode::Input, M>,
    pub in3: PinAsync<'a, crate::mode::Input, M>,
    pub in4: PinAsync<'a, crate::mode::Input, M>,
    pub in5: PinAsync<'a, crate::mode::Input, M>,
    pub in6: PinAsync<'a, crate::mode::Input, M>,
    pub in7: PinAsync<'a, crate::mode::Input, M>,

    /// Must be called from your real hardware interrupt to wake tasks.
    pub interrupts: InterruptHandler<'a, M>,
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
    type Error = B::BusError;

    /// PCA9702 is input-only, return an error here.
    fn set(&mut self, _mask_high: u32, _mask_low: u32) -> Result<(), Self::Error> {
        panic!("PCA9702 is input-only, cannot set output states");
    }

    /// PCA9702 is input-only, return an error here.
    fn is_set(&mut self, _mask_high: u32, _mask_low: u32) -> Result<u32, Self::Error> {
        panic!("PCA9702 is input-only, cannot read back output states");
    }

    /// Read the actual input bits from the PCA9702 device
    fn get(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error> {
        let val = self.bus.read_inputs()? as u32;
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
        let mut spi_mock = SpiMock::new(&expectations);
        let mut pca = Pca9702::new(spi_mock.clone());
        let pins = pca.split();

        // For each call, the driver re-reads from SPI, returning 0xA5 each time.
        // 0xA5 = 0b10100101 => in0=1, in1=0, in2=1, in3=0, in4=0, in5=1, in6=0, in7=1
        assert_eq!(pins.in0.is_high().unwrap(), true); // LSB => 1
        assert_eq!(pins.in1.is_high().unwrap(), false);
        assert_eq!(pins.in2.is_high().unwrap(), true);

        spi_mock.done();
    }

    #[test]
    #[should_panic]
    fn pca9702_output_fails() {
        let spi_mock = SpiMock::new(&[]);
        let mut pca = Pca9702::new(spi_mock);
        let pins = pca.split();

        pins.in0.access_port_driver(|drv| {
            drv.set(0x01, 0x00).unwrap_err();
        });
    }
}
