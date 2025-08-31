// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]
#![no_main]

// rp235x-hal: Support for the RP235x Boot ROM's "Block" structures
// pub mod block;

// We have to do this if we don't otherwise use it to ensure its vector table
// gets linked in.
use rp235x_pac::OTP_DATA_RAW;

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
#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: ImageDefBlock = ImageDefBlock {
    marker_start: 0xffffded3,
    item: 0x10210142,
    length: 0x000001ff,
    offset: 0x00000000,
    marker_end: 0xab123579,
};

pub fn reset_secure_boot() {
    let crit1 = unsafe { OTP_DATA_RAW::steal() };
    crit1.crit1().reset();
}

#[entry]
fn main() -> ! {
    reset_secure_boot();
    // Default boot speed, until we bother raising it:
    const CYCLES_PER_MS: u32 = 8_000;

    unsafe { hubris_kern::startup::start_kernel(CYCLES_PER_MS) }
}
