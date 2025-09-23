use hubris_kern::klog;
use riscv::asm;
use rp235x_adc::{adc_config_pads, adc_enable, adc_read_raw};

pub fn setup_adc(adc_pin: usize) {
    adc_config_pads(adc_pin);
    adc_enable();
}

pub fn run_adc_demo() {
    const ADC_PIN: usize = 26;

    for i in 0..5 {
        let raw_adc = adc_read_raw(ADC_PIN);
        klog!("ADC Reading [{}]: {}", i, raw_adc);
        // Wait for a while
        asm::delay(1_20_30_000);
    }
}
