use riscv::asm;
use rp235x_gpio::{led_config_gpio, led_config_io, led_config_pads, led_set};

pub fn run_demo() {
    const LED_PIN: usize = 22;

    // Initialize GPIO22
    led_config_gpio(LED_PIN);
    led_config_pads(LED_PIN);
    led_config_io(LED_PIN);

    for i in 0..6 {
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
