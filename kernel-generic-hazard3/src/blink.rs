use riscv::asm;
use rp235x_gpio::{pico_led_init, pico_led_set};

pub fn run_demo() {
    const LED_PIN: usize = 22;

    // Initialize GPIO22 as output
    let gpio_peripherals = pico_led_init(LED_PIN);

    for i in 0..6 {
        // Turn LED on
        pico_led_set(&gpio_peripherals, LED_PIN, true);
        // Wait for a while
        asm::delay(1_20_30_000);
        // Turn LED off
        pico_led_set(&gpio_peripherals, LED_PIN, false);
        // Wait for a while
        asm::delay(1_20_30_000);
    }
}
