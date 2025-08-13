//! # GPIO 'Blinky' Example
//!
//! This application demonstrates how to control a GPIO pin on the rp235x.
//!
//! It may need to be adapted to your particular board layout and/or pin assignment.
//!
//! See the `Cargo.toml` file for Copyright and license details.

#![no_std]
#![no_main]

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use userlib as _;

// Alias for our GPIO crate
use rp235x_gpio::{pico_led_init, pico_led_set};

#[export_name = "main"]
fn main() -> ! {
    const LED_PIN: usize = 22;

    // Initialize GPIO22 as output
    let gpio_peripherals = pico_led_init(LED_PIN);

    loop {
        // Turn LED on
        pico_led_set(&gpio_peripherals, LED_PIN, true);
    }
}
