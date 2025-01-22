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

use crate::common::PortDriver;
use crate::mode::HasInput;
use crate::mutex::PortMutex;
use crate::pin::{Pin as SyncPin, PinError};
use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicU16, Ordering};
use core::task::{Context, Poll, Waker};
use embedded_hal::digital::ErrorType;
use embedded_hal_async::digital::Wait;
use heapless::Vec;

/// Maximum number of tasks that can wait on a single pin's events.
/// Increase this if you expect more concurrency.
pub const MAX_WAKERS_PER_PIN: usize = 4;

static NEXT_WAITER_ID: AtomicU16 = AtomicU16::new(1);

/// Conditions for which a future might be waiting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitCondition {
    High,
    Low,
    RisingEdge,
    FallingEdge,
    AnyEdge,
}

impl WaitCondition {
    /// For levels (High/Low), is it satisfied by the current known state of the pin?
    ///
    /// For edge conditions, we never say "yes" up front. We want the *next* edge.
    fn is_satisfied_immediately(self, current_pin_state: bool) -> bool {
        match self {
            WaitCondition::High => current_pin_state,
            WaitCondition::Low => !current_pin_state,
            WaitCondition::RisingEdge => false,
            WaitCondition::FallingEdge => false,
            WaitCondition::AnyEdge => false,
        }
    }

    /// Does this condition match a transition from (was_high) to (is_high)?
    fn matches_edge(self, was_high: bool, is_high: bool) -> bool {
        let rising = !was_high && is_high;
        let falling = was_high && !is_high;

        match self {
            WaitCondition::High => is_high, // for an immediate "becomes high" check
            WaitCondition::Low => !is_high, // for an immediate "becomes low" check
            WaitCondition::RisingEdge => rising,
            WaitCondition::FallingEdge => falling,
            WaitCondition::AnyEdge => rising || falling,
        }
    }
}

/// A wait registration for one task: which condition is awaited and the task's waker.
#[derive(Debug)]
struct PinWaiter {
    id: u16,
    condition: WaitCondition,
    waker: Waker,
}

/// Shared, interrupt-driven async state for a single port-expander chip.
/// - Tracks last-known state (bitmask) of up to 32 pins
/// - Maintains waker lists for each pin
pub struct AsyncPortState {
    pub last_known_state: u32,
    waiters: [Vec<PinWaiter, MAX_WAKERS_PER_PIN>; 32],
}

impl AsyncPortState {
    pub fn new() -> Self {
        Self {
            last_known_state: 0,
            waiters: Default::default(),
        }
    }
}

impl Default for AsyncPortState {
    fn default() -> Self {
        Self::new()
    }
}

/// Use this in your actual interrupt routine. It compares the new pin states
/// vs. the old, wakes any tasks that match the changes, and updates
/// `last_known_state`.
pub struct InterruptHandler<'a, M>
where
    M: PortMutex,
    M::Port: PortDriver,
{
    port_mutex: &'a M,
    async_state: &'a RefCell<AsyncPortState>,
}

impl<'a, M> InterruptHandler<'a, M>
where
    M: PortMutex,
    M::Port: PortDriver,
{
    /// Construct a new `InterruptHandler`. Store it or pass it into your hardware ISR.
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

                // We'll remove from the list any waiters whose condition is satisfied
                // by the transition (was_high -> is_high).
                let waiters_for_pin = &mut st.waiters[pin_idx];
                let mut i = 0;
                while i < waiters_for_pin.len() {
                    let cond = waiters_for_pin[i].condition;
                    if cond.matches_edge(was_high, is_high) {
                        let w = waiters_for_pin.remove(i);
                        w.waker.wake();
                    } else {
                        i += 1;
                    }
                }
            }
        }

        // Update the stored state
        st.last_known_state = new_state;
        Ok(())
    }
}

/// Asynchronous pin object implementing [`embedded_hal_async::digital::Wait`].
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

impl<'a, MODE, M> ErrorType for PinAsync<'a, MODE, M>
where
    MODE: HasInput,
    M: PortMutex,
    M::Port: PortDriver,
    <M::Port as PortDriver>::Error: core::fmt::Debug,
{
    type Error = PinError<<M::Port as PortDriver>::Error>;
}

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
        WaitForCondition::new(self.pin_index, self.async_state, WaitCondition::High)
            .await
            .expect("Infallible");
        Ok(())
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        if self.is_low()? {
            return Ok(());
        }
        WaitForCondition::new(self.pin_index, self.async_state, WaitCondition::Low)
            .await
            .expect("Infallible");
        Ok(())
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        WaitForCondition::new(self.pin_index, self.async_state, WaitCondition::RisingEdge)
            .await
            .expect("Infallible");
        Ok(())
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        WaitForCondition::new(self.pin_index, self.async_state, WaitCondition::FallingEdge)
            .await
            .expect("Infallible");
        Ok(())
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        WaitForCondition::new(self.pin_index, self.async_state, WaitCondition::AnyEdge)
            .await
            .expect("Infallible");
        Ok(())
    }
}

/// The internal future type used by `PinAsync` wait methods. Once it registers
/// a waker, it stays Pending until the interrupt handler removes and wakes it.
///
/// **Edge conditions** always wait for a *future* event.  
/// **Level conditions** will short‐circuit if the current known state is already
/// satisfied, otherwise they wait for the next time the interrupt handler sees
/// that pin become that level (which is effectively a “level or edge”).
struct WaitForCondition<'s> {
    pin_index: u8,
    async_state: &'s RefCell<AsyncPortState>,
    condition: WaitCondition,
    id: u16,

    /// Have we already inserted ourselves into the waiters list?
    registered: bool,
    /// Did we see that we are "done" (removed) during a wake?
    done: bool,
}

impl<'s> WaitForCondition<'s> {
    fn new(
        pin_index: u8,
        async_state: &'s RefCell<AsyncPortState>,
        condition: WaitCondition,
    ) -> Self {
        // Generate a new ID atomically
        let id = NEXT_WAITER_ID.fetch_add(1, Ordering::Relaxed);

        Self {
            pin_index,
            async_state,
            condition,
            id,
            registered: false,
            done: false,
        }
    }
}

impl<'s> Future for WaitForCondition<'s> {
    // Once we've performed the initial synchronous check, no more I/O occurs in poll(),
    // so we can't produce a bus error. We use `PinError<core::convert::Infallible>`.
    type Output = Result<(), PinError<core::convert::Infallible>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = self.get_mut();

        // If we already determined "done", return Ready
        if me.done {
            return Poll::Ready(Ok(()));
        }

        let mut state = me.async_state.borrow_mut();
        let mask = 1 << me.pin_index;
        let current_pin_state = (state.last_known_state & mask) != 0;
        let pin_waiters = &mut state.waiters[me.pin_index as usize];

        // If this is a level condition (High/Low), check if it’s already satisfied
        // by the current known state. If so, we can immediately return Ready.
        // (For edges, we want *future* transitions, so do NOT short‐circuit.)
        if !me.registered && me.condition.is_satisfied_immediately(current_pin_state) {
            me.done = true;
            return Poll::Ready(Ok(()));
        }

        // Otherwise we need to be in the waiter list, so we can be woken
        // by the interrupt that sees the next transition or next time
        // the pin becomes the desired level.

        // Check if we are still in the list. If not, it means we got woken
        // by the ISR (interrupt) which removed us. We must be done.
        let pos = pin_waiters.iter().position(|pw| pw.id == me.id);

        match (me.registered, pos) {
            // Not registered yet => insert ourselves
            (false, None) => {
                // Attempt push
                if pin_waiters.len() == pin_waiters.capacity() {
                    panic!("No waker slots left");
                }
                pin_waiters
                    .push(PinWaiter {
                        id: me.id,
                        condition: me.condition,
                        waker: cx.waker().clone(),
                    })
                    .expect("push must succeed due to capacity check");
                me.registered = true;
                // We remain Pending
                Poll::Pending
            }

            // We are registered, but the ISR removed us => we must have been triggered => done
            (true, None) => {
                me.done = true;
                Poll::Ready(Ok(()))
            }

            // We are still in the list => update waker if changed, remain Pending
            (_, Some(idx)) => {
                let pw = &mut pin_waiters[idx];
                if !pw.waker.will_wake(cx.waker()) {
                    pw.waker = cx.waker().clone();
                }
                Poll::Pending
            }
        }
    }
}

impl<'s> Drop for WaitForCondition<'s> {
    /// If the future is dropped before it is satisfied, remove from the list (if present).
    fn drop(&mut self) {
        let mut st = self.async_state.borrow_mut();
        let waiters = &mut st.waiters[self.pin_index as usize];

        if let Some(pos) = waiters.iter().position(|pw| pw.id == self.id) {
            waiters.remove(pos);
        }
    }
}
