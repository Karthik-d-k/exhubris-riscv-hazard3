//! RP235x GPIO helper library

#![no_std]

use rp235x_pac::io_bank0::gpio::gpio_ctrl::FUNCSEL_A;
use rp235x_pac::{IO_BANK0, SIO};

pub fn pico_led_init(led_pin: usize) {
    let sio: rp235x_pac::SIO = unsafe { SIO::steal() };
    let io_bank0: rp235x_pac::IO_BANK0 = unsafe { IO_BANK0::steal() };
    let mask = 1u32 << led_pin as u32;
    // set dir as INPUT
    sio.gpio_oe_clr().write(|w| unsafe { w.bits(mask) });
    // Drive GPIO low
    sio.gpio_out_clr().write(|w| unsafe { w.bits(mask) });
    // Set GPIO function to SIO
    unsafe {
        io_bank0
            .gpio(led_pin)
            .gpio_ctrl()
            .write_with_zero(|w| w.funcsel().variant(FUNCSEL_A::SIO));
    }
    // set dir as OUTPUT
    sio.gpio_oe_set().write(|w| unsafe { w.bits(mask) });
}

pub fn pico_set_led(led_pin: usize, led_on: bool) {
    let sio: rp235x_pac::SIO = unsafe { SIO::steal() };
    let mask = 1u32 << led_pin as u32;
    if led_on {
        sio.gpio_out_set().write(|w| unsafe { w.bits(mask) });
    } else {
        sio.gpio_out_clr().write(|w| unsafe { w.bits(mask) });
    }
}
