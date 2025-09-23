use riscv::asm;
use rp235x_gpio::{led_config_gpio, led_config_io, led_config_pads, led_set};

pub fn setup_led(led_pin: usize) {
    // Initialize GPIO for the specified LED pin
    led_config_gpio(led_pin);
    led_config_pads(led_pin);
    led_config_io(led_pin);
}

pub fn run_led_demo() {
    const LED_PIN: usize = 22;

    for _ in 0..5 {
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
