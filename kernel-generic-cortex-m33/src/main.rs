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

    p.RESETS.reset().modify(|_, w| w.io_bank0().clear_bit());
    while !p.RESETS.reset_done().read().io_bank0().bit() {}

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

    unsafe { hubris_kern::startup::start_kernel(cycles_per_ms) }
}
