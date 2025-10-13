//! RP235x PWM helper library

use rp235x_pac::io_bank0::gpio::gpio_ctrl::FUNCSEL_A;
use rp235x_pac::{IO_BANK0, PADS_BANK0, PWM};

/// Configure default config for the pwm slice
#[inline(always)]
pub fn pwm_set_default_config(pwm: &PWM, pwm_id: usize) {
    // free-running mode
    pwm.ch(pwm_id).csr().modify(|_, w| w.divmode().div());
    // write_ph_correct(false);
    pwm.ch(pwm_id)
        .csr()
        .modify(|_, w| w.ph_correct().bit(false));
    // write_div_int(1); // No divisor
    pwm.ch(pwm_id)
        .div()
        .modify(|_, w| unsafe { w.int().bits(1) });
    // write_div_frac(0); // No divisor
    pwm.ch(pwm_id)
        .div()
        .modify(|_, w| unsafe { w.frac().bits(0) });
    // write_inv_a(false); //Don't invert the channel
    pwm.ch(pwm_id).csr().modify(|_, w| w.a_inv().bit(false));
    // write_inv_b(false); //Don't invert the channel
    pwm.ch(pwm_id).csr().modify(|_, w| w.b_inv().bit(false));
    // write_top(0xfffe); // Wrap at 0xfffe, so cc = 0xffff can indicate 100% duty cycle
    pwm.ch(pwm_id)
        .top()
        .write(|w| unsafe { w.top().bits(0xfffe) });
    // write_ctr(0x0000); //Reset the counter
    pwm.ch(pwm_id)
        .ctr()
        .write(|w| unsafe { w.ctr().bits(0x0000) });
    // write_cc_a(0); //Default duty cycle of 0%
    pwm.ch(pwm_id).cc().modify(|_, w| unsafe { w.a().bits(0) });
    // write_cc_b(0); //Default duty cycle of 0%
    pwm.ch(pwm_id).cc().modify(|_, w| unsafe { w.b().bits(0) });
}

/// Set PWM phase-correct mode
#[inline(always)]
pub fn pwm_set_ph_correct(pwm: &PWM, pwm_id: usize) {
    pwm.ch(pwm_id).csr().modify(|_, w| w.ph_correct().bit(true));
}

/// Enable PWM
#[inline(always)]
pub fn pwm_enable(pwm: &PWM, pwm_id: usize) {
    pwm.ch(pwm_id).csr().modify(|_, w| w.en().bit(true));
}

/// Configure PWM function
#[inline(always)]
pub fn pwm_config_io(io_bank0: &IO_BANK0, pads_bank0: &PADS_BANK0, pwm_pin: usize) {
    pads_bank0.gpio(pwm_pin).modify(|_, w| {
        // Set input enable on, output disable off
        // RP2350: input enable defaults to off, so this is important!
        w.ie().set_bit();
        w.od().clear_bit();
        w
    });

    unsafe {
        io_bank0
            .gpio(pwm_pin)
            .gpio_ctrl()
            .write_with_zero(|w| w.funcsel().variant(FUNCSEL_A::PWM));
    };

    pads_bank0.gpio(pwm_pin).modify(|_, w| {
        // RP2350: remove pad isolation now a function is wired up
        w.iso().clear_bit();
        w
    });
}

/// Set PWM duty cycle channel A
#[inline(always)]
pub fn pwm_set_duty_cycle_a(pwm: &PWM, pwm_id: usize, duty_cycle: u16) {
    pwm.ch(pwm_id)
        .cc()
        .modify(|_, w| unsafe { w.a().bits(duty_cycle) }); // write_cc_a
}

/// Set PWM duty cycle channel B
#[inline(always)]
pub fn pwm_set_duty_cycle_b(pwm: &PWM, pwm_id: usize, duty_cycle: u16) {
    pwm.ch(pwm_id)
        .cc()
        .modify(|_, w| unsafe { w.b().bits(duty_cycle) }); // write_cc_b
}

/// Get PWM max duty cycle
#[inline(always)]
pub fn pwm_get_max_duty_cycle(pwm: &PWM, pwm_id: usize) -> u16 {
    pwm.ch(pwm_id).top().read().top().bits().saturating_add(1)
}
