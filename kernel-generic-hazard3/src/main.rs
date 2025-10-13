// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]
#![no_main]

mod adc;
mod blink;
mod pwm;

use hubris_kern::klog;

// We have to do this if we don't otherwise use it to ensure its vector table
// gets linked in.
use rp235x_pac as _;

use rp235x_sys::{adc::adc_clock_enable, resets_sys};

// use crate::block::ImageDef;
use riscv_rt::entry;

/// A Block as understood by the Boot ROM.
///
/// This is an Image Definition Block
///
/// It contains within the special start and end markers the Boot ROM is looking for.
#[derive(Debug)]
#[repr(C)]
pub struct ImageDefBlock {
    marker_start: u32,
    item: u32,
    length: u32,
    offset: u32,
    marker_end: u32,
}

/// Tell the Boot ROM about our application
/// Refer RP2350 Datasheet, Section: 5.9.5.2. Minimum RISC-V IMAGE_DEF
#[link_section = ".image_def"]
#[used]
pub static IMAGMINIMUM_RISCV_IMAGE_DEF: ImageDefBlock = ImageDefBlock {
    marker_start: 0xffffded3,
    item: 0x11010142,
    length: 0x000001ff,
    offset: 0x00000000,
    marker_end: 0xab123579,
};

// Pin configuration constants
#[cfg(feature = "kernel-task-blinky")]
const LED_PIN: usize = 22; // GP22

#[cfg(feature = "kernel-task-adc-blinky")]
const ADC_PIN: usize = 26; // ADC0
#[cfg(feature = "kernel-task-adc-blinky")]
const PWM_PIN: usize = 0; // GP0
#[cfg(feature = "kernel-task-adc-blinky")]
const PWM_ID: usize = 0; // GPIO0 --> PWM-0A

#[entry]
fn main() -> ! {
    let p = unsafe { rp235x_pac::Peripherals::steal() };

    // enable adc clock
    // this should be done 1st before lifting ADC from resets, orelse ADC doesn't come out of resets
    adc_clock_enable(&p.CLOCKS);

    // wait for system peripherals to be ready
    resets_sys(&p.RESETS);
    klog!("SYSTEM PERIPHERALS ARE READY");

    // Provide U-mode access for riscv and non-secure mode access for arm for all GPIOs
    // This may not work on pico 2(W) boards, refer: Errata `RP2350-E3`
    // this shouldn't be done , somehow pwm::run_pwm_demo doesn't run
    // if U-mode access is enabled
    #[cfg(feature = "kernel-task-blinky")]
    {
        p.ACCESSCTRL
            .gpio_nsmask0()
            .write(|w| unsafe { w.bits(0xFFFF_FFFF) });
    }

    // TODO fix/update this for RP2350
    let cycles_per_ms = if p.CLOCKS.clk_sys_ctrl().read().src().is_clk_ref() {
        // This is the reset state, so we'll assume we launched directly from
        // flash running on the ROSC.
        6_000 // ish
    } else {
        // This is _not_ the reset state, so we'll assume that the pico-debug
        // resident debugger has reconfigured things to run off the 48 MHz USB
        // clock.
        48_000
    };

    klog!("Cycles per ms: {} MHz", cycles_per_ms / 1000);

    // M-mode setup for ADC and GPIO
    #[cfg(feature = "kernel-task-blinky")]
    {
        blink::setup_led(&p.SIO, &p.IO_BANK0, &p.PADS_BANK0, LED_PIN);
    }
    #[cfg(feature = "kernel-task-adc-blinky")]
    {
        adc::setup_adc(&p.ADC, &p.IO_BANK0, &p.PADS_BANK0, ADC_PIN);
        pwm::setup_pwm(&p.PWM, &p.IO_BANK0, &p.PADS_BANK0, PWM_PIN);
    }

    // This is used to prove that M-mode GPIO works fine, but U-mode doesn't due to Errata `RP2350-E3`
    #[cfg(feature = "kernel-task-blinky")]
    {
        klog!("Running LED demo in Kernel (M) Mode");
        // Executing in M-mode.
        blink::run_led_demo(&p.SIO, LED_PIN);
    }

    #[cfg(feature = "kernel-task-adc-blinky")]
    {
        klog!("Running ADC-PWM demo in Kernel (M) Mode");
        adc::run_adc_demo(&p.ADC, ADC_PIN);
        pwm::run_pwm_demo(&p.PWM, PWM_ID);
        // pwm::run_adc_pwm_demo(&p.ADC, &p.PWM, ADC_PIN, PWM_ID);
    }

    unsafe { hubris_kern::startup::start_kernel(cycles_per_ms) }
}
