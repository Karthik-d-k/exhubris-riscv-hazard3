// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]
#![no_main]

// rp235x-hal: Support for the RP235x Boot ROM's "Block" structures
// pub mod block;

// We have to do this if we don't otherwise use it to ensure its vector table
// gets linked in.
use rp235x_pac::{CLOCKS, RESETS};

// use crate::block::ImageDef;
use riscv_rt::entry;

use rp235x_gpio::{
    enable_clock, reset_bring_down_io_bank0, reset_bring_down_pads_bank0, reset_bring_up_io_bank0,
    reset_bring_up_pads_bank0,
};

use rtt_target::{rprintln, rtt_init_print};

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
#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: ImageDefBlock = ImageDefBlock {
    marker_start: 0xffffded3,
    item: 0x11010142,
    length: 0x000001ff,
    offset: 0x00000000,
    marker_end: 0xab123579,
};

#[entry]
fn main() -> ! {
    // rtt_init_print!();
    // enable clock
    let clock = unsafe { CLOCKS::steal() };
    enable_clock(&clock);
    rprintln!("Clock enabled");

    // bring IO_BANK0 and PADS_BANK0 out of reset
    let resets = unsafe { RESETS::steal() };
    reset_bring_down_io_bank0(&resets);
    reset_bring_down_pads_bank0(&resets);
    rprintln!("Bringing down io and pads");
    reset_bring_up_io_bank0(&resets);
    reset_bring_up_pads_bank0(&resets);
    rprintln!("Bringing up io and pads");

    // Default boot speed, until we bother raising it:
    const CYCLES_PER_MS: u32 = 8_000;

    rprintln!("Starting kernel");
    unsafe { hubris_kern::startup::start_kernel(CYCLES_PER_MS) }
}
