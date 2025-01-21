#[cfg(feature = "async")]
use core::future::Future;
#[cfg(feature = "async")]
use core::pin::Pin;
#[cfg(feature = "async")]
use core::task::{Context, Poll, Waker};

#[cfg(feature = "async")]
use heapless::Vec;

#[cfg(feature = "async")]
use core::cell::RefCell;

#[cfg(feature = "async")]
use crate::common::PortDriver;
#[cfg(feature = "async")]
use crate::mode::HasInput;
#[cfg(feature = "async")]
use crate::mutex::PortMutex;
#[cfg(feature = "async")]
use crate::pin::{Pin as SyncPin, PinError};
#[cfg(feature = "async")]
use embedded_hal::digital::ErrorType;
#[cfg(feature = "async")]
use embedded_hal_async::digital::Wait;

/// Maximum number of tasks that can wait on a single pin's events.
#[cfg(feature = "async")]
const MAX_WAKERS_PER_PIN: usize = 4;

/// Possible wait conditions that a future might be waiting for.
#[cfg(feature = "async")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitCondition {
    High,
    Low,
    RisingEdge,
    FallingEdge,
    AnyEdge,
}

/// A single wait registration: which condition is the task waiting for, plus the Waker.
#[cfg(feature = "async")]
#[derive(Debug)]
pub struct PinWaiter {
    pub condition: WaitCondition,
    pub waker: Waker,
}

/// Shared, interrupt-driven async state for a single port-expander chip.
/// - Tracks last-known state (bitmask) of up to 32 pins
/// - Maintains waker lists for each pin
#[cfg(feature = "async")]
pub struct AsyncPortState {
    pub last_known_state: u32,
    pub waiters: [Vec<PinWaiter, MAX_WAKERS_PER_PIN>; 32],
}

#[cfg(feature = "async")]
impl AsyncPortState {
    pub fn new() -> Self {
        // Each of the 32 slots has an empty waker Vec
        Self {
            last_known_state: 0,
            waiters: Default::default(),
        }
    }
}

/// The `InterruptHandler` is what you call when the hardware interrupt pin
/// from the port-expander fires. It checks which pins changed, wakes tasks
/// that were waiting for that change, and updates the `last_known_state`.
#[cfg(feature = "async")]
pub struct InterruptHandler<'a, M>
where
    M: PortMutex,
    M::Port: PortDriver,
{
    port_mutex: &'a M,
    async_state: &'a RefCell<AsyncPortState>,
}

#[cfg(feature = "async")]
impl<'a, M> InterruptHandler<'a, M>
where
    M: PortMutex,
    M::Port: PortDriver,
{
    pub fn new(port_mutex: &'a M, async_state: &'a RefCell<AsyncPortState>) -> Self {
        Self {
            port_mutex,
            async_state,
        }
    }

    /// Call this from your actual interrupt handler. It reads the current state
    /// of all pins from the driver, compares with the old state, and wakes any tasks
    /// that were waiting for these transitions.
    pub fn handle_interrupts(&self) -> Result<(), <M::Port as PortDriver>::Error> {
        // Acquire the driver to read all pin states
        let new_state = self.port_mutex.lock(|drv| {
            // We read 32 bits. For an 8/16-bit expander, you'd adapt accordingly,
            // but typically you'd do a "get(0xFFFF_FFFF, 0)" to read which pins are high.
            drv.get(0xFFFF_FFFF, 0)
        })?;

        let mut st = self.async_state.borrow_mut();
        let old_state = st.last_known_state;
        let changed = old_state ^ new_state;

        if changed == 0 {
            // No changes, so no tasks to wake
            return Ok(());
        }

        // For each pin that changed, determine if it rose or fell, and wake tasks
        for pin_idx in 0..32 {
            let mask = 1 << pin_idx;
            if (changed & mask) != 0 {
                // This pin changed
                let was_high = (old_state & mask) != 0;
                let is_high = (new_state & mask) != 0;

                let is_rising = !was_high && is_high;
                let is_falling = was_high && !is_high;

                let waiters = &mut st.waiters[pin_idx];
                let mut i = 0;
                while i < waiters.len() {
                    let cond = waiters[i].condition;
                    let wake_now = if is_rising {
                        matches!(
                            cond,
                            WaitCondition::High
                                | WaitCondition::RisingEdge
                                | WaitCondition::AnyEdge
                        )
                    } else if is_falling {
                        matches!(
                            cond,
                            WaitCondition::Low
                                | WaitCondition::FallingEdge
                                | WaitCondition::AnyEdge
                        )
                    } else {
                        false
                    };

                    if wake_now {
                        let w = waiters.remove(i);
                        w.waker.wake();
                    } else {
                        i += 1;
                    }
                }
            }
        }

        // Update last known
        st.last_known_state = new_state;
        Ok(())
    }
}

/// An asynchronous pin type which implements [`embedded_hal_async::digital::Wait`].
#[cfg(feature = "async")]
pub struct PinAsync<'a, MODE, M>
where
    MODE: HasInput,
    M: PortMutex,
    M::Port: PortDriver,
{
    /// The underlying "synchronous" pin reference (same port driver).
    pub sync_pin: SyncPin<'a, MODE, M>,

    /// Reference to the shared async state for this entire port.
    pub async_state: &'a RefCell<AsyncPortState>,

    /// Which bit/pin index in the port. 0 => least-significant bit, etc.
    pin_index: u8,
}

#[cfg(feature = "async")]
impl<'a, MODE, M> PinAsync<'a, MODE, M>
where
    MODE: HasInput,
    M: PortMutex,
    M::Port: PortDriver,
{
    pub fn new(
        sync_pin: SyncPin<'a, MODE, M>,
        async_state: &'a RefCell<AsyncPortState>,
        pin_index: u8,
    ) -> Self {
        Self {
            sync_pin,
            async_state,
            pin_index,
        }
    }

    /// **Synchronous** check if this pin is currently high.
    /// Equivalent to `Pin::is_high()`.
    pub fn is_high(&self) -> Result<bool, PinError<<M::Port as PortDriver>::Error>> {
        self.sync_pin.is_high()
    }

    /// **Synchronous** check if this pin is currently low.
    /// Equivalent to `Pin::is_low()`.
    pub fn is_low(&self) -> Result<bool, PinError<<M::Port as PortDriver>::Error>> {
        self.sync_pin.is_low()
    }
}

#[cfg(feature = "async")]
impl<'a, MODE, M> ErrorType for PinAsync<'a, MODE, M>
where
    MODE: HasInput,
    M: PortMutex,
    M::Port: PortDriver,
    <M::Port as PortDriver>::Error: core::fmt::Debug,
{
    type Error = PinError<<M::Port as PortDriver>::Error>;
}

// The main trick: We do *not* store `&mut self` inside the future. Instead,
// we create a small future that only references the minimal shared state.
// That avoids the self-referential lifetime problem.
#[cfg(feature = "async")]
impl<'a, MODE, M> Wait for PinAsync<'a, MODE, M>
where
    MODE: HasInput,
    M: PortMutex,
    M::Port: PortDriver,
    <M::Port as PortDriver>::Error: core::fmt::Debug,
{
    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        // If already high, return immediately
        if self.is_high()? {
            return Ok(());
        }
        Ok(WaitForCondition {
            pin_index: self.pin_index,
            async_state: self.async_state,
            condition: WaitCondition::High,
            registered: false,
        }
        .await
        .unwrap())
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        if self.is_low()? {
            return Ok(());
        }
        WaitForCondition {
            pin_index: self.pin_index,
            async_state: self.async_state,
            condition: WaitCondition::Low,
            registered: false,
        }
        .await
        .map_err(|_| unreachable!())
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        // Always wait for an actual transition
        WaitForCondition {
            pin_index: self.pin_index,
            async_state: self.async_state,
            condition: WaitCondition::RisingEdge,
            registered: false,
        }
        .await
        .map_err(|_| unreachable!())
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        WaitForCondition {
            pin_index: self.pin_index,
            async_state: self.async_state,
            condition: WaitCondition::FallingEdge,
            registered: false,
        }
        .await
        .map_err(|_| unreachable!())
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        WaitForCondition {
            pin_index: self.pin_index,
            async_state: self.async_state,
            condition: WaitCondition::AnyEdge,
            registered: false,
        }
        .await
        .map_err(|_| unreachable!())
    }
}

/// The internal future type for any pin-wait operation.
/// We keep track of whether we've already registered a waker in the `AsyncPortState`.
#[cfg(feature = "async")]
struct WaitForCondition<'s> {
    pin_index: u8,
    async_state: &'s RefCell<AsyncPortState>,
    condition: WaitCondition,
    registered: bool,
}

#[cfg(feature = "async")]
impl<'s> Future for WaitForCondition<'s> {
    type Output = Result<(), PinError<core::convert::Infallible>>;
    // ^ we use `PinError<Infallible>` because
    //   after we've done the initial is_high/is_low check,
    //   no more driver calls are made in the future (the driver calls happen in `handle_interrupts`).
    //   So there's no I/O error from the device in the poll steps.
    //   If you prefer, you can define your own "NoError" type or keep it flexible.
    //   Typically we'd store the driver error type if we do repeated reads in poll,
    //   but our design does not do that.

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = self.get_mut();

        // Once we've put a waker in the waiters array, we are guaranteed
        // that the interrupt handler will wake it up after a relevant edge or state change.
        // So we only need to register once. Then we stay in Poll::Pending until woken.
        if !me.registered {
            let mut st = me.async_state.borrow_mut();
            let pin_waiters = &mut st.waiters[me.pin_index as usize];

            if pin_waiters.len() == pin_waiters.capacity() {
                // In no-std + no-heap scenarios, we must either fail or panic.
                panic!("No waker slots left for pin {}", me.pin_index);
            }

            pin_waiters
                .push(PinWaiter {
                    condition: me.condition,
                    waker: cx.waker().clone(),
                })
                .expect("push should succeed due to capacity check");
            me.registered = true;
        }

        // Not ready yet => wait for interrupt
        Poll::Pending
    }
}
