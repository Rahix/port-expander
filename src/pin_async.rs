//! Asynchronous pin-waiting support for port-expanders, using embedded-hal-async's
//! [`digital::Wait`] trait.  
//!
//! This module is only built if the `"async"` feature is enabled. It provides:
//! 1. A shared [`AsyncPortState`] which tracks last-known pin states and holds waiters.
//! 2. An [`InterruptHandler`] to call from your real hardware interrupt routine.
//! 3. A [`PinAsync`] type implementing `embedded_hal_async::digital::Wait`.
//!
//! **Concurrency caution**: If your interrupt can fire while tasks are registering
//! new wakers (i.e. calling `wait_for_*`), you must ensure no double borrowing of
//! `AsyncPortState`. For example, wrap it (and the driver) in a critical-section
//! or the same mutex. Failing to do so can cause runtime panics in no-std.

#[cfg(feature = "async")]
use core::cell::RefCell;
#[cfg(feature = "async")]
use core::future::Future;
#[cfg(feature = "async")]
use core::pin::Pin;
#[cfg(feature = "async")]
use core::task::{Context, Poll, Waker};

#[cfg(feature = "async")]
use heapless::Vec;

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
/// Increase this if you expect more concurrency.
#[cfg(feature = "async")]
pub const MAX_WAKERS_PER_PIN: usize = 4;

/// Conditions for which a future might be waiting.
#[cfg(feature = "async")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitCondition {
    High,
    Low,
    RisingEdge,
    FallingEdge,
    AnyEdge,
}

/// A wait registration for one task: which condition is awaited and the task's waker.
#[cfg(feature = "async")]
#[derive(Debug)]
struct PinWaiter {
    condition: WaitCondition,
    waker: Waker,
}

/// Shared, interrupt-driven async state for a single port-expander chip.
/// - Tracks last-known state (bitmask) of up to 32 pins
/// - Maintains waker lists for each pin
#[cfg(feature = "async")]
pub struct AsyncPortState {
    pub last_known_state: u32,
    waiters: [Vec<PinWaiter, MAX_WAKERS_PER_PIN>; 32],
}

#[cfg(feature = "async")]
impl AsyncPortState {
    pub fn new() -> Self {
        Self {
            last_known_state: 0,
            waiters: Default::default(),
        }
    }
}

/// Use this in your actual interrupt routine. It compares the new pin states
/// vs. the old, wakes any tasks that match the changes, and updates
/// `last_known_state`.
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
    /// Construct a new `InterruptHandler`. Typically you store this somewhere
    /// or pass it into your hardware IRQ routine.
    pub fn new(port_mutex: &'a M, async_state: &'a RefCell<AsyncPortState>) -> Self {
        Self {
            port_mutex,
            async_state,
        }
    }

    /// Called from your hardware ISR. Reads the new pin states, compares with old,
    /// wakes tasks that match, updates `last_known_state`.
    pub fn handle_interrupts(&self) -> Result<(), <M::Port as PortDriver>::Error> {
        // Read the current state. For an 8- or 16-bit expander, you'd typically do
        // something like get(0xFF,0) or get(0xFFFF,0). Using 0xFFFF_FFFF to read “all 32 pins”
        // is a general approach if the driver supports up to 32.
        let new_state = self.port_mutex.lock(|drv| drv.get(0xFFFF_FFFF, 0))?;

        let mut st = self.async_state.borrow_mut();
        let old_state = st.last_known_state;
        let changed = old_state ^ new_state;

        if changed == 0 {
            // Nothing changed; no tasks to wake.
            return Ok(());
        }

        // For each pin that changed, figure out if it rose or fell.
        for pin_idx in 0..32 {
            let mask = 1 << pin_idx;
            if (changed & mask) != 0 {
                let was_high = (old_state & mask) != 0;
                let is_high = (new_state & mask) != 0;

                let is_rising = !was_high && is_high;
                let is_falling = was_high && !is_high;

                let waiters = &mut st.waiters[pin_idx];
                let mut i = 0;
                // Remove+wake any waiters whose condition is now satisfied.
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

        // Update the stored state.
        st.last_known_state = new_state;
        Ok(())
    }
}

/// Asynchronous pin object implementing [`embedded_hal_async::digital::Wait`].
#[cfg(feature = "async")]
pub struct PinAsync<'a, MODE, M>
where
    MODE: HasInput,
    M: PortMutex,
    M::Port: PortDriver,
{
    /// The underlying synchronous pin reference.
    sync_pin: SyncPin<'a, MODE, M>,
    /// Reference to the shared async state for the entire port.
    async_state: &'a RefCell<AsyncPortState>,
    /// Which pin index (0..31).
    pin_index: u8,
}

#[cfg(feature = "async")]
impl<'a, MODE, M> PinAsync<'a, MODE, M>
where
    MODE: HasInput,
    M: PortMutex,
    M::Port: PortDriver,
{
    /// Constructs a `PinAsync` from a sync pin plus a reference to the shared `AsyncPortState`.
    /// The `pin_index` must match the bit number used in the underlying driver.
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

    /// Check synchronously if this pin is currently high.
    pub fn is_high(&self) -> Result<bool, PinError<<M::Port as PortDriver>::Error>> {
        self.sync_pin.is_high()
    }

    /// Check synchronously if this pin is currently low.
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
        WaitForCondition {
            pin_index: self.pin_index,
            async_state: self.async_state,
            condition: WaitCondition::High,
            registered: false,
        }
        .await
        .map_err(|_| unreachable!())
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

/// The internal future type used by `PinAsync` wait methods. Once it registers
/// a waker, it stays Pending until the interrupt handler removes and wakes it.
#[cfg(feature = "async")]
struct WaitForCondition<'s> {
    pin_index: u8,
    async_state: &'s RefCell<AsyncPortState>,
    condition: WaitCondition,
    registered: bool,
}

#[cfg(feature = "async")]
impl<'s> Future for WaitForCondition<'s> {
    // Once we've performed the initial synchronous check, no more I/O occurs in poll(),
    // so we can't produce a bus error. We use `PinError<core::convert::Infallible>`.
    type Output = Result<(), PinError<core::convert::Infallible>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = self.get_mut();

        // Only register once. The interrupt handler will wake us eventually.
        if !me.registered {
            let mut st = me.async_state.borrow_mut();
            let pin_waiters = &mut st.waiters[me.pin_index as usize];

            if pin_waiters.len() == pin_waiters.capacity() {
                // panic is the only recourse if we run out of slots.
                panic!("No waker slots left for pin {}", me.pin_index);
            }

            pin_waiters
                .push(PinWaiter {
                    condition: me.condition,
                    waker: cx.waker().clone(),
                })
                .expect("push must succeed due to capacity check");
            me.registered = true;

            // Re-check if the condition might already be satisfied
            // based on the last-known state in `st`.  This closes the race
            // between the initial synchronous check and waker registration.
            let mask = 1 << me.pin_index;
            let is_high = (st.last_known_state & mask) != 0;
            match me.condition {
                WaitCondition::High if is_high => return Poll::Ready(Ok(())),
                WaitCondition::Low if !is_high => return Poll::Ready(Ok(())),
                // RisingEdge / FallingEdge / AnyEdge => we specifically want *future* transitions,
                // so we do NOT return ready just for the pin currently being high or low.
                _ => {}
            }
        }

        // We wait to be woken by `handle_interrupts()`.
        Poll::Pending
    }
}
