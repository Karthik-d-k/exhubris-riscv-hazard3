//! RP235x System Library
//!
//! This library provides hardware abstraction for the RP235x microcontroller,
//! including GPIO, ADC, and PWM functionality.

#![no_std]

pub mod adc;
pub mod gpio;
pub mod pwm;

use rp235x_pac::RESETS;

#[inline(always)]
/// Reset the system, wait for system peripherals to be ready
pub fn resets_sys(resets: &RESETS) {
    // reset bringup PADS_BANK0
    resets.reset().modify(|_, w| w.pads_bank0().clear_bit());
    while !resets.reset_done().read().pads_bank0().bit() {}

    // reset bringup IO_BANK0
    resets.reset().modify(|_, w| w.io_bank0().clear_bit());
    while !resets.reset_done().read().io_bank0().bit() {}

    // reset bringup PWM
    resets.reset().modify(|_, w| w.pwm().clear_bit());
    while !resets.reset_done().read().pwm().bit() {}

    // reset bringup ADC
    resets.reset().modify(|_, w| w.adc().clear_bit());
    while !resets.reset_done().read().adc().bit() {}
}
