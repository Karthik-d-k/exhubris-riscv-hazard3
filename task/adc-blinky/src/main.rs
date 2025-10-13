//! # GPIO 'Blinky' Example
//!
//! This application demonstrates how to control an external LED and adjusts LED brightness using MAX4466 Microphone.

#![no_std]
#![no_main]

#[cfg(target_arch = "riscv32")]
use riscv::asm;

#[cfg(target_arch = "arm")]
use cortex_m::asm;

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use userlib as _;

use rp235x_pac::{ADC, PWM};
use rp235x_sys::adc::sample_adc_window;
use rp235x_sys::pwm::pwm_set_duty_cycle_a;

#[export_name = "main"]
fn main() -> ! {
    let adc = unsafe { ADC::steal() };
    let pwm = unsafe { PWM::steal() };

    const LOW: u16 = 0;
    const HIGH: u16 = 65535;
    const ADC_PIN: usize = 26; // ADC0
    const PWM_PIN: usize = 0; // GPIO0
    const PWM_ID: usize = PWM_PIN; // GPIO0 --> PWM-0A

    // // Infinite loop, fading LED up and down
    // loop {
    //     // Ramp brightness up
    //     for i in LOW..=HIGH {
    //         asm::delay(2400); // 50us -> 48MHz
    //         pwm_set_duty_cycle_a(&pwm, PWM_ID, i);
    //     }

    //     // Ramp brightness down
    //     for i in (LOW..=HIGH).rev() {
    //         asm::delay(2400); // 50us -> 48MHz
    //         pwm_set_duty_cycle_a(&pwm, PWM_ID, i);
    //     }

    //     asm::delay(48_000_000); // 1s -> 48MHz
    // }

    loop {
        let (signal_min, signal_max, peak_to_peak, dc) = sample_adc_window(&adc, ADC_PIN);
        // pwm_set_duty_cycle_a(&pwm, PWM_ID, dc);
        asm::delay(24_000_000); // 0.5s -> 48MHz
    }
}
