//! RP235x ADC helper library

use rp235x_pac::clocks::clk_adc_ctrl::AUXSRC_A;
use rp235x_pac::io_bank0::gpio::gpio_ctrl::OEOVER_A;
use rp235x_pac::{ADC, CLOCKS, IO_BANK0, PADS_BANK0};

/// Configure the pin to be used with an ADC and disables its digital circuitry.
#[inline(always)]
pub fn adc_config_pads(io_bank0: &IO_BANK0, pads_bank0: &PADS_BANK0, adc_pin: usize) {
    // // set function as SIO (not needed for ADC !!?)
    // unsafe {
    //     io_bank0
    //         .gpio(adc_pin)
    //         .gpio_ctrl()
    //         .write_with_zero(|w| w.funcsel().variant(FUNCSEL_A::SIO));
    // };

    // disable the pin’s digital functions
    let (pue, pde) = (false, false);
    pads_bank0.gpio(adc_pin).modify(|_, w| {
        w.pue().bit(pue).pde().bit(pde); // set pulltype to pullnone
        w.ie().bit(false); // set_input_enable(false)
        w.od().bit(true); // set_output_disable(true)
        w
    });

    unsafe {
        io_bank0
            .gpio(adc_pin)
            .gpio_ctrl()
            .write_with_zero(|w| w.oeover().variant(OEOVER_A::DISABLE))
    };
}

#[inline(always)]
fn adc_wait_ready(adc: &ADC) {
    while !adc.cs().read().ready().bit_is_set() {
        core::hint::spin_loop();
    }
}

#[inline(always)]
pub fn adc_clock_enable(clocks: &CLOCKS) {
    clocks.clk_adc_ctrl().modify(|_, w| {
        w.auxsrc().variant(AUXSRC_A::CLKSRC_PLL_USB); // use USB clock (48MHz)
        w.enable().set_bit(); // enable clock
        w
    });
}

#[inline(always)]
pub fn adc_enable(adc: &ADC) {
    // Enable adc
    adc.cs().write(|w| w.en().set_bit());
    adc_wait_ready(&adc);
}

/// Read the raw ADC counts.
pub fn adc_read_raw(adc: &ADC, adc_pin: usize) -> u16 {
    let chan: u8 = (adc_pin - 26) as u8; // GPIO26 -> 0 ... GPIO29 -> 3 (ADC channels)

    // wait until adc conversion is complete
    adc_wait_ready(&adc);
    // start a new conversion
    adc.cs()
        .modify(|_, w| unsafe { w.ainsel().bits(chan).start_once().set_bit() });
    // wait until adc conversion is complete
    adc_wait_ready(&adc);

    // return the most recently sampled ADC value
    adc.result().read().result().bits()
}

/// Listen to ADC for short interval, recording min, max, p2p and duty cycle
pub fn sample_adc_window(adc: &ADC, adc_pin: usize) -> (u16, u16, u16, u16) {
    let mut signal_min = u16::MAX;
    let mut signal_max = 0u16;
    // At 48MHz, 33ms = 1,584,000 cycles
    // Each ADC read takes 96 cycles, so max reads = 1,584,000 / 96 = 16,500
    const SAMPLES_PER_WINDOW: usize = 16_000;

    // Take continuous samples during the 33ms window (no delays needed)
    for _ in 0..SAMPLES_PER_WINDOW {
        let signal = adc_read_raw(adc, adc_pin);

        if signal < signal_min {
            signal_min = signal;
        }
        if signal > signal_max {
            signal_max = signal;
        }
        // No delay needed - ADC read itself takes 96 cycles
    }

    let peak_to_peak = signal_max - signal_min; // Audio amplitude

    // Remove low-level noise, boost
    let dc = ((peak_to_peak as i32 - 250) * 4).clamp(0, 65535) as u16;

    (signal_min, signal_max, peak_to_peak, dc)
}
