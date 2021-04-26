pub fn write_multiple<PD, MUTEX, MODE: crate::mode::HasOutput, const N: usize>(
    mut pins: [&mut crate::Pin<'_, MODE, MUTEX>; N],
    states: [bool; N],
) -> Result<(), PD::Error>
where
    PD: crate::PortDriver,
    MUTEX: shared_bus::BusMutex<Bus = PD>,
{
    let mut mask_set_high = 0x00;
    let mut mask_set_low = 0x00;

    for (pin, state) in pins.iter_mut().zip(states.iter()) {
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

pub fn read_multiple<PD, MUTEX, MODE: crate::mode::HasInput, const N: usize>(
    pins: [&crate::Pin<'_, MODE, MUTEX>; N],
) -> Result<[bool; N], PD::Error>
where
    PD: crate::PortDriver,
    MUTEX: shared_bus::BusMutex<Bus = PD>,
{
    let mask = pins.iter().map(|p| p.pin_mask()).fold(0, |m, p| m | p);
    let mask_in = pins[0].port_driver().lock(|drv| drv.get(mask, 0))?;

    let mut ret = [false; N];
    for (pin, state) in pins.iter().zip(ret.iter_mut()) {
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
}
