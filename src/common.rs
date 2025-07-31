pub trait PortDriver {
    type Error;

    /// Set all pins in `mask_high` to HIGH and all pins in `mask_low` to LOW.
    ///
    /// The driver should implements this such that all pins change state at the same time.
    fn set(&mut self, mask_high: u32, mask_low: u32) -> Result<(), Self::Error>;

    /// Check whether pins in `mask_high` were set HIGH and pins in `mask_low` were set LOW.
    ///
    /// For each pin in either of the masks, the returned `u32` should have a 1 if they meet the
    /// expected state and a 0 otherwise.  All other bits MUST always stay 0.
    ///
    /// If a bit is set in both `mask_high` and `mask_low`, the resulting bit must be 1.
    fn is_set(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error>;

    /// Check whether pins in `mask_high` are driven HIGH and pins in `mask_low` are driven LOW.
    ///
    /// For each pin in either of the masks, the returned `u32` should have a 1 if they meet the
    /// expected state and a 0 otherwise.  All other bits MUST always stay 0.
    ///
    /// If a bit is set in both `mask_high` and `mask_low`, the resulting bit must be 1.
    fn get(&mut self, mask_high: u32, mask_low: u32) -> Result<u32, Self::Error>;

    fn toggle(&mut self, mask: u32) -> Result<(), Self::Error> {
        // for all pins which are currently low, make them high.
        let mask_high = self.is_set(0, mask)?;
        // for all pins which are currently high, make them low.
        let mask_low = self.is_set(mask, 0)?;
        self.set(mask_high, mask_low)
    }
}

pub trait PortDriverTotemPole: PortDriver {
    /// Set the direction for all pins in `mask` to direction `dir`.
    ///
    /// To prevent electrical glitches, when making pins outputs, the `state` can be either `true`
    /// or `false` to immediately put the pin HIGH or LOW upon switching.
    fn set_direction(&mut self, mask: u32, dir: Direction, state: bool) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Input,
    Output,
}

pub trait PortDriverPolarity: PortDriver {
    /// Set the polarity of all pins in `mask` either `inverted` or not.
    fn set_polarity(&mut self, mask: u32, inverted: bool) -> Result<(), Self::Error>;
}

pub trait PortDriverPullDown: PortDriver {
    /// Enable pull-downs for pins in mask or set the pin to floating if enable is false.
    fn set_pull_down(&mut self, mask: u32, enable: bool) -> Result<(), Self::Error>;
}

pub trait PortDriverPullUp: PortDriver {
    /// Enable pull-ups for pins in mask or set the pin to floating if enable is false.
    fn set_pull_up(&mut self, mask: u32, enable: bool) -> Result<(), Self::Error>;
}

pub trait PortDriverInterrupts: PortDriver {
    /// Fetch the interrupt status of pins from the port expander.
    ///
    /// This method should fetch the interrupt information from the port expander and clear the
    /// remote registers.  The values need to be stored locally as part of the port driver.
    ///
    /// The local values should be amended by new interrupt information instead of overwriting.
    fn fetch_interrupt_state(&mut self) -> Result<(), Self::Error>;

    /// Read whether pins changed state since the last interrupt.
    ///
    /// This method should only query the locally cached values that were retrieved by
    /// `fetch_interrupt_state()`.
    ///
    /// This method should reset the locally cached pin-change status for pins from the mask.
    fn query_pin_change(&mut self, mask: u32) -> u32;
}

pub trait PortDriverIrqMask: PortDriver {
    /// Set/clear the interrupt mask of the port expander.
    fn set_interrupt_mask(&mut self, mask_set: u32, mask_clear: u32) -> Result<(), Self::Error>;
}

pub trait PortDriverIrqState: PortDriver {
    /// Read the state of pins from the last interrupt.
    ///
    /// This method returns a tuple:
    /// 1. The mask of pins that actually changed state. Value must be the same that would have
    ///    been returned by `query_pin_change()`.
    /// 2. The state of each of the pins in the changed mask.
    ///
    /// This method should only query the locally cached values that were retrieved by
    /// `fetch_interrupt_state()`.
    ///
    /// This method should reset the locally cached pin-change status for pins from the mask.
    fn query_interrupt_state(&mut self, mask: u32) -> (u32, u32);
}

/// Pin Modes
pub mod mode {
    /// Trait for pin-modes which can be used to set a logic level.
    pub trait HasOutput {}
    /// Trait for pin-modes which can be used to read a logic level.
    pub trait HasInput {}

    /// Pin configured as an input.
    pub struct Input;
    impl HasInput for Input {}

    /// Pin configured as an output.
    pub struct Output;
    impl HasOutput for Output {}

    /// Pin configured as a quasi-bidirectional input/output.
    pub struct QuasiBidirectional;
    impl HasInput for QuasiBidirectional {}
    impl HasOutput for QuasiBidirectional {}
}
