/// Set multiple pins at the same time.
///
/// The usual method of setting multiple pins
///
/// ```no_run
/// # let i2c = embedded_hal_mock::i2c::Mock::new(&[]);
/// # let mut pcf = port_expander::Pcf8574::new(i2c, false, false, false);
/// # let p = pcf.split();
/// # let mut io0 = p.p0;
/// # let mut io1 = p.p1;
/// io0.set_high().unwrap();
/// io1.set_low().unwrap();
/// ```
///
/// can be problematic because the time between the two operations might be significant (they are
/// done as two separate bus transactions).  If it is desired that multiple pins change state in a
/// single bus transaction, the `write_multiple()` function provides an interface to do this.
///
/// ## Example
/// ```no_run
/// # let i2c = embedded_hal_mock::i2c::Mock::new(&[]);
/// # let mut pcf = port_expander::Pcf8574::new(i2c, false, false, false);
/// # let p = pcf.split();
/// # let mut io0 = p.p0;
/// # let mut io1 = p.p1;
/// port_expander::write_multiple(
///     [&mut io0, &mut io1],
///     [true, false],
/// ).unwrap();
/// ```
pub fn write_multiple<PD, MUTEX, MODE: crate::mode::HasOutput, const N: usize>(
    pins: [&mut crate::Pin<'_, MODE, MUTEX>; N],
    states: [bool; N],
) -> Result<(), PD::Error>
where
    PD: crate::PortDriver,
    MUTEX: shared_bus::BusMutex<Bus = PD>,
{
    let mut mask_set_high = 0x00;
    let mut mask_set_low = 0x00;

    let port_driver = pins[0].port_driver();
    for (pin, state) in pins.iter().zip(states.iter()) {
        assert!(core::ptr::eq(pin.port_driver(), port_driver));
        if *state {
            mask_set_high |= pin.pin_mask();
        } else {
            mask_set_low |= pin.pin_mask();
        }
    }

    pins[0].port_driver().lock(|drv| {
        drv.set(mask_set_high, mask_set_low)?;
        Ok(())
    })
}

/// Read multiple pins at the same time.
///
/// When a port-expander sends an interrupt that one of its inputs changed state, it might be
/// important to find out which input was responsible as quickly as possible _and_ by checking all
/// inputs at once.  The naive approach of checking the pins in order
///
/// ```no_run
/// # let i2c = embedded_hal_mock::i2c::Mock::new(&[]);
/// # let mut pcf = port_expander::Pcf8574::new(i2c, false, false, false);
/// # let p = pcf.split();
/// # let io0 = p.p0;
/// # let io1 = p.p1;
/// if io0.is_high().unwrap() {
///     // ...
/// } else if io1.is_high().unwrap() {
///     // ...
/// }
/// ```
///
/// is suboptimal because each read will happen as its own bus transaction and there is thus quite
/// some delay.  Also the pins are checked one after the other, not all at once, which could lead
/// to glitches.  The `read_multiple()` function provides an interface to circumvent these
/// problems.
///
/// ## Example
/// ```no_run
/// # let i2c = embedded_hal_mock::i2c::Mock::new(&[]);
/// # let mut pcf = port_expander::Pcf8574::new(i2c, false, false, false);
/// # let p = pcf.split();
/// # let io0 = p.p0;
/// # let io1 = p.p1;
/// let values = port_expander::read_multiple([&io0, &io1]).unwrap();
/// if values[0] {
///     // ...
/// } else if values[1] {
///     // ...
/// }
/// ```
pub fn read_multiple<PD, MUTEX, MODE: crate::mode::HasInput, const N: usize>(
    pins: [&crate::Pin<'_, MODE, MUTEX>; N],
) -> Result<[bool; N], PD::Error>
where
    PD: crate::PortDriver,
    MUTEX: shared_bus::BusMutex<Bus = PD>,
{
    let mask = pins.iter().map(|p| p.pin_mask()).fold(0, |m, p| m | p);
    let port_driver = pins[0].port_driver();
    let mask_in = port_driver.lock(|drv| drv.get(mask, 0))?;

    let mut ret = [false; N];
    for (pin, state) in pins.iter().zip(ret.iter_mut()) {
        assert!(core::ptr::eq(pin.port_driver(), port_driver));
        *state = mask_in & pin.pin_mask() != 0;
    }

    Ok(ret)
}

#[cfg(test)]
mod tests {
    use embedded_hal_mock::i2c as mock_i2c;

    #[test]
    fn pcf8574_write_multiple() {
        let expectations = [
            // single writes for multiple pins
            mock_i2c::Transaction::write(0x21, vec![0b10111011]),
            mock_i2c::Transaction::write(0x21, vec![0b10101111]),
        ];
        let mut bus = mock_i2c::Mock::new(&expectations);

        let mut pcf = crate::Pcf8574::new(bus.clone(), true, false, false);
        let mut pcf_pins = pcf.split();

        super::write_multiple(
            [&mut pcf_pins.p2, &mut pcf_pins.p4, &mut pcf_pins.p6],
            [false, true, false],
        )
        .unwrap();

        super::write_multiple([&mut pcf_pins.p2, &mut pcf_pins.p4], [true, false]).unwrap();

        bus.done();
    }

    #[test]
    fn pcf8575_write_multiple() {
        let expectations = [
            // single writes for multiple pins
            mock_i2c::Transaction::write(0x21, vec![0b10111011, 0b11011101]),
            mock_i2c::Transaction::write(0x21, vec![0b10101111, 0b11010111]),
        ];
        let mut bus = mock_i2c::Mock::new(&expectations);

        let mut pcf = crate::Pcf8575::new(bus.clone(), true, false, false);
        let mut pcf_pins = pcf.split();

        super::write_multiple(
            [
                &mut pcf_pins.p02,
                &mut pcf_pins.p04,
                &mut pcf_pins.p06,
                &mut pcf_pins.p11,
                &mut pcf_pins.p13,
                &mut pcf_pins.p15,
            ],
            [false, true, false, false, true, false],
        )
        .unwrap();

        super::write_multiple(
            [
                &mut pcf_pins.p02,
                &mut pcf_pins.p04,
                &mut pcf_pins.p11,
                &mut pcf_pins.p13,
            ],
            [true, false, true, false],
        )
        .unwrap();

        bus.done();
    }

    #[test]
    fn pca9536_read_multiple() {
        let expectations = [
            // single reads for multiple pins
            mock_i2c::Transaction::write_read(0x41, vec![0x00], vec![0b00000101]),
            mock_i2c::Transaction::write_read(0x41, vec![0x00], vec![0b00001010]),
        ];
        let mut bus = mock_i2c::Mock::new(&expectations);

        let mut pca = crate::Pca9536::new(bus.clone());
        let pca_pins = pca.split();

        let res = super::read_multiple([&pca_pins.io0, &pca_pins.io1, &pca_pins.io2]).unwrap();
        assert_eq!(res, [true, false, true]);

        let res = super::read_multiple([&pca_pins.io1, &pca_pins.io0, &pca_pins.io3]).unwrap();
        assert_eq!(res, [true, false, true]);

        bus.done();
    }

    #[test]
    #[should_panic]
    fn pca9538_multiple_assert_same_chip() {
        let expectations = [
            // single reads for multiple pins
            mock_i2c::Transaction::write_read(0x70, vec![0x00], vec![0b00000101]),
        ];
        let mut bus = mock_i2c::Mock::new(&expectations);

        let mut pca0 = crate::Pca9538::new(bus.clone(), false, false);
        let pca0_pins = pca0.split();
        let mut pca1 = crate::Pca9538::new(bus.clone(), false, true);
        let pca1_pins = pca1.split();

        let _ = super::read_multiple([&pca0_pins.io0, &pca1_pins.io1]);

        bus.done();
    }
}
