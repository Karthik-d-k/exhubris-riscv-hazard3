//! # GPIO 'Blinky' Example
//!
//! This application demonstrates how to control a GPIO pin to blink on the rp235x.

#![no_std]
#![no_main]

use riscv::asm;

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use userlib as _;

use rp235x_pac::SIO;

// our system crate
use rp235x_sys::gpio::led_set;

#[export_name = "main"]
fn main() -> ! {
    let sio = unsafe { SIO::steal() };
    const LED_PIN: usize = 22; // GP22

    loop {
        // Turn LED on
        led_set(&sio, LED_PIN, true);
        // Wait for a while
        asm::delay(24_000_000); // 0.5s -> 48MHz
                                // Turn LED off
        led_set(&sio, LED_PIN, false);
        // Wait for a while
        asm::delay(24_000_000); // 0.5s -> 48MHz
    }
}
