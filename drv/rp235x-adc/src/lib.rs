//! RP235x ADC helper library

#![no_std]

use rp235x_pac::clocks::clk_adc_ctrl::AUXSRC_A;
use rp235x_pac::io_bank0::gpio::gpio_ctrl::OEOVER_A;
use rp235x_pac::{ADC, CLOCKS, IO_BANK0, PADS_BANK0, RESETS};

/// Configure the pin to be used with an ADC and disables its digital circuitry.
pub fn adc_config_pads(adc_pin: usize) {
    let io_bank0 = unsafe { IO_BANK0::steal() };
    let pads_bank0 = unsafe { PADS_BANK0::steal() };

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

fn adc_wait_ready(adc: &ADC) {
    while !adc.cs().read().ready().bit_is_set() {
        core::hint::spin_loop();
    }
}

pub fn adc_clock_enable() {
    let clocks = unsafe { CLOCKS::steal() };

    clocks.clk_adc_ctrl().modify(|_, w| {
        w.auxsrc().variant(AUXSRC_A::CLKSRC_PLL_USB); // use USB clock (48MHz)
        w.enable().set_bit(); // enable clock
        w
    });
}

pub fn adc_enable() {
    // enable adc clock
    adc_clock_enable();
    let adc = unsafe { ADC::steal() };
    let resets = unsafe { RESETS::steal() };

    // reset bringup ADC
    resets.reset().modify(|_, w| w.adc().clear_bit());
    while !resets.reset_done().read().adc().bit() {}

    // Enable adc
    adc.cs().write(|w| w.en().set_bit());
    adc_wait_ready(&adc);
}

/// Read the raw ADC counts.
pub fn adc_read_raw(adc_pin: usize) -> u16 {
    let adc = unsafe { ADC::steal() };
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
