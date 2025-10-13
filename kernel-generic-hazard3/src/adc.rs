use hubris_kern::klog;
use rp235x_pac::{ADC, IO_BANK0, PADS_BANK0};
use rp235x_sys::adc::{adc_config_pads, adc_enable, sample_adc_window};

#[inline(always)]
pub fn setup_adc(adc: &ADC, io_bank0: &IO_BANK0, pads_bank0: &PADS_BANK0, adc_pin: usize) {
    adc_config_pads(io_bank0, pads_bank0, adc_pin);
    adc_enable(adc);
}

pub fn run_adc_demo(adc: &ADC, adc_pin: usize) {
    for i in 0..5 {
        let (signal_min, signal_max, peak_to_peak, dc) = sample_adc_window(adc, adc_pin);

        klog!(
            "ADC Reading [{}]: min={}, max={}, peak_to_peak={}, duty_cycle={}",
            i,
            signal_min,
            signal_max,
            peak_to_peak,
            dc
        );
    }
}
