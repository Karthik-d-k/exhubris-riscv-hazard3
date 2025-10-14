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
    // This may not work on pico 2(W) boards, refer: Errata `RP2350-E3`, works only for SIO, bit not for ADC/PWM
    p.ACCESSCTRL
        .gpio_nsmask0()
        .write(|w| unsafe { w.bits(0xFFFF_FFFF) });

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
    const LED_PIN: usize = 22; // GP22
    blink::setup_led(&p.SIO, &p.IO_BANK0, &p.PADS_BANK0, LED_PIN);

    // This is used to prove that M-mode GPIO works fine, but U-mode doesn't due to Errata `RP2350-E3`

    klog!("Running LED demo in Kernel (M) Mode");
    // Executing in M-mode.
    blink::run_led_demo(&p.SIO, LED_PIN);

    unsafe { hubris_kern::startup::start_kernel(cycles_per_ms) }
}
