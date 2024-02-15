/// Common interface for mutex implementations.
///
/// `port-expander` needs a mutex to ensure only a single pin object can access the port-expander at the same time,
/// in concurrent situations.  `port-expander` already implements this trait for a number of existing
/// mutex types.  Most of them are guarded by a feature that needs to be enabled.  Here is an
/// overview:
///
/// | Mutex | Feature Name | Notes |
/// | --- | --- | --- |
/// | [`core::cell::RefCell`] | _always available_ | For sharing within a single execution context. |
/// | [`std::sync::Mutex`][mutex-std] | `std` | For platforms where `std` is available. |
///
/// [mutex-std]: https://doc.rust-lang.org/std/sync/struct.Mutex.html
///
/// For other mutex types, a custom implementation is needed.  Due to the orphan rule, it might be
/// necessary to wrap it in a newtype.  As an example, this is what such a custom implementation
/// might look like:
///
/// ```
/// struct MyMutex<T>(std::sync::Mutex<T>);
///
/// impl<T> port_expander::PortMutex for MyMutex<T> {
///     type Port = T;
///
///     fn create(v: T) -> Self {
///         Self(std::sync::Mutex::new(v))
///     }
///
///     fn lock<R, F: FnOnce(&mut Self::Port) -> R>(&self, f: F) -> R {
///         let mut v = self.0.lock().unwrap();
///         f(&mut v)
///     }
/// }
/// ```
pub trait PortMutex {
    /// The actual port-expander that is wrapped inside this mutex.
    type Port;

    /// Create a new mutex of this type.
    fn create(v: Self::Port) -> Self;

    /// Lock the mutex and give a closure access to the port-expander inside.
    fn lock<R, F: FnOnce(&mut Self::Port) -> R>(&self, f: F) -> R;
}

impl<T> PortMutex for core::cell::RefCell<T> {
    type Port = T;

    fn create(v: Self::Port) -> Self {
        core::cell::RefCell::new(v)
    }

    fn lock<R, F: FnOnce(&mut Self::Port) -> R>(&self, f: F) -> R {
        let mut v = self.borrow_mut();
        f(&mut v)
    }
}

#[cfg(any(test, feature = "std"))]
impl<T> PortMutex for std::sync::Mutex<T> {
    type Port = T;

    fn create(v: Self::Port) -> Self {
        std::sync::Mutex::new(v)
    }

    fn lock<R, F: FnOnce(&mut Self::Port) -> R>(&self, f: F) -> R {
        let mut v = self.lock().unwrap();
        f(&mut v)
    }
}
