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
use rp235x_adc as _;

#[export_name = "main"]
fn main() -> ! {
    loop {
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }
}
