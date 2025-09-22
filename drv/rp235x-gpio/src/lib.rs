//! RP235x GPIO helper library

#![no_std]

use rp235x_pac::io_bank0::gpio::gpio_ctrl::FUNCSEL_A;
use rp235x_pac::{CLOCKS, IO_BANK0, PADS_BANK0, RESETS, SIO};

/// Enable Clock
pub fn enable_clock(clock: &CLOCKS) {
    clock.clk_peri_ctrl().modify(|_, w| w.enable().set_bit());
}

/// Bring up IO_BANK0
pub fn reset_bring_up_io_bank0(resets: &RESETS) {
    resets.reset().modify(|_, w| w.io_bank0().clear_bit());
    while resets.reset_done().read().io_bank0().bit_is_clear() {}
}

/// Bring down IO_BANK0
pub fn reset_bring_down_io_bank0(resets: &RESETS) {
    resets.reset().modify(|_, w| w.io_bank0().set_bit());
}

/// Bring up PADS_BANK0
pub fn reset_bring_up_pads_bank0(resets: &RESETS) {
    resets.reset().modify(|_, w| w.pads_bank0().clear_bit());
    while resets.reset_done().read().pads_bank0().bit_is_clear() {}
}

/// Bring down PADS_BANK0
pub fn reset_bring_down_pads_bank0(resets: &RESETS) {
    resets.reset().modify(|_, w| w.pads_bank0().set_bit());
}

pub fn led_config_gpio(led_pin: usize) {
    let sio = unsafe { SIO::steal() };
    let mask = 1u32 << led_pin as u32;
    // Set GPIO as output
    sio.gpio_oe_set().write(|w| unsafe { w.bits(mask) });
}

pub fn led_config_pads(led_pin: usize) {
    let pads_bank0 = unsafe { PADS_BANK0::steal() };
    // Configure pad settings
    pads_bank0.gpio(led_pin).modify(|_, w| {
        // Set input enable on, output disable off
        // RP2350: input enable defaults to off, so this is important!
        w.ie().set_bit();
        w.od().clear_bit();
        // RP2350: remove pad isolation now a function is wired up
        w.iso().clear_bit();
        w
    });
}

/// Zero all fields apart from fsel; we want this IO to do what the peripheral tells it.
/// This doesn't affect e.g. pullup/pulldown, as these are in pad controls.
pub fn led_config_io(led_pin: usize) {
    let io_bank0 = unsafe { IO_BANK0::steal() };
    unsafe {
        io_bank0
            .gpio(led_pin)
            .gpio_ctrl()
            .write_with_zero(|w| w.funcsel().variant(FUNCSEL_A::SIO));
    };
}

pub fn led_set(led_pin: usize, led_on: bool) {
    let sio = unsafe { SIO::steal() };
    let mask = 1u32 << led_pin as u32;
    if led_on {
        sio.gpio_out_set().write(|w| unsafe { w.bits(mask) });
    } else {
        sio.gpio_out_clr().write(|w| unsafe { w.bits(mask) });
    }
}
