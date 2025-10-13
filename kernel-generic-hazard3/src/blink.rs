use riscv::asm;
use rp235x_pac::{IO_BANK0, PADS_BANK0, SIO};
use rp235x_sys::gpio::{led_config_io, led_config_output, led_set};

#[inline(always)]
pub fn setup_led(sio: &SIO, io_bank0: &IO_BANK0, pads_bank0: &PADS_BANK0, led_pin: usize) {
    // Initialize GPIO for the specified LED pin
    led_config_io(io_bank0, pads_bank0, led_pin);
    led_config_output(sio, led_pin);
}

pub fn run_led_demo(sio: &SIO, led_pin: usize) {
    for _ in 0..5 {
        // Turn LED on
        led_set(sio, led_pin, true);
        // Wait for a while
        asm::delay(1_20_30_000);
        // Turn LED off
        led_set(sio, led_pin, false);
        // Wait for a while
        asm::delay(1_20_30_000);
    }
}
