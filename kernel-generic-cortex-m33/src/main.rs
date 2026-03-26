// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]
#![no_main]

// We have to do this if we don't otherwise use it to ensure its vector table
// gets linked in.
use rp235x_pac as _;

// use crate::block::ImageDef;
use cortex_m_rt::entry;

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
/// Refer RP2350 Datasheet, 5.9.5.1. Minimum Arm IMAGE_DEF
/// TODO: Assuming CRIT1.SECURE_BOOT_ENABLE is clear
#[link_section = ".image_def"]
#[used]
pub static MINIMUM_ARM_IMAGE_DEF: ImageDefBlock = ImageDefBlock {
    marker_start: 0xffffded3,
    item: 0x10210142,
    length: 0x000001ff,
    offset: 0x00000000,
    marker_end: 0xab123579,
};

#[entry]
fn main() -> ! {
    let p = unsafe { rp235x_pac::Peripherals::steal() };

    // Start XOSC (12 MHz crystal)
    p.XOSC.ctrl().write(|w| {
        w.freq_range()
            .variant(rp235x_pac::xosc::ctrl::FREQ_RANGE_A::_1_15MHZ)
    });
    p.XOSC.startup().write(|w| unsafe { w.delay().bits(47) });
    p.XOSC
        .ctrl()
        .write(|w| w.enable().variant(rp235x_pac::xosc::ctrl::ENABLE_A::ENABLE));
    while !p.XOSC.status().read().stable().bit_is_set() {}

    // Route XOSC (12 MHz) to clk_peri
    p.CLOCKS.clk_peri_ctrl().write(|w| {
        unsafe { w.auxsrc().bits(4) }; // xosc_clksrc
        w.enable().set_bit();
        w
    });

    // TODO fix/update this for RP2350
    let cycles_per_ms = 12_000;

    unsafe { hubris_kern::startup::start_kernel(cycles_per_ms) }
}
