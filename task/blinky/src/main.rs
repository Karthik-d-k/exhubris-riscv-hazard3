//! # GPIO 'Blinky' Example
//!
//! This application demonstrates how to control a GPIO pin to blink on the rp235x.

#![no_std]
#![no_main]

#[cfg(target_arch = "riscv32")]
use riscv::asm;

#[cfg(target_arch = "arm")]
use cortex_m::asm;

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use userlib as _;

// our GPIO crate
use rp235x_gpio::led_set;

#[export_name = "main"]
fn main() -> ! {
    const LED_PIN: usize = 22;

    loop {
        // Turn LED on
        led_set(LED_PIN, true);
        // Wait for a while
        asm::delay(1_20_30_000);
        // Turn LED off
        led_set(LED_PIN, false);
        // Wait for a while
        asm::delay(1_20_30_000);
    }
}
