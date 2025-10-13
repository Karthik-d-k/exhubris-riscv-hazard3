use hubris_kern::klog;
use riscv::asm;
use rp235x_pac::{ADC, IO_BANK0, PADS_BANK0, PWM};
use rp235x_sys::adc::sample_adc_window;
use rp235x_sys::pwm::{
    pwm_config_io, pwm_enable, pwm_set_default_config, pwm_set_duty_cycle_a, pwm_set_ph_correct,
};

#[inline(always)]
pub fn setup_pwm(pwm: &PWM, io_bank0: &IO_BANK0, pads_bank0: &PADS_BANK0, pwm_pin: usize) {
    // TODO, extend the functionality to support all GPIOs
    let pwm_id = 0usize; // GPIO0 --> PWM-0A

    pwm_set_default_config(pwm, pwm_id);
    pwm_set_ph_correct(pwm, pwm_id);
    pwm_enable(pwm, pwm_id);
    pwm_config_io(io_bank0, pads_bank0, pwm_pin);
}

pub fn run_pwm_demo(pwm: &PWM, pwm_id: usize) {
    const LOW: u16 = 0;
    const HIGH: u16 = 65535;

    klog!("Running PWM demo");

    // fading LED up and down
    for _ in 0..2 {
        // Ramp brightness up
        for j in LOW..=HIGH {
            asm::delay(480); // 10us -> 48MHz
            pwm_set_duty_cycle_a(pwm, pwm_id, j);
        }

        // Ramp brightness down
        for j in (LOW..=HIGH).rev() {
            asm::delay(480); // 10us -> 48MHz
            pwm_set_duty_cycle_a(pwm, pwm_id, j);
        }

        asm::delay(48_000_000); // 1s -> 48MHz
    }
}

pub fn run_adc_pwm_demo(adc: &ADC, pwm: &PWM, adc_pin: usize, pwm_id: usize) {
    let mut i = 0;

    klog!("Running PWM demo");

    while i < 2 {
        let (signal_min, signal_max, peak_to_peak, dc) = sample_adc_window(adc, adc_pin);

        klog!(
            "ADC Reading [{}]: min={}, max={}, peak_to_peak={}, duty_cycle={}",
            i,
            signal_min,
            signal_max,
            peak_to_peak,
            dc
        );
        klog!("Setting PWM duty cycle [{}]: {}", i, dc);
        pwm_set_duty_cycle_a(pwm, pwm_id, dc);
        asm::delay(1_20_30_000);
        i += 1;
    }
}
