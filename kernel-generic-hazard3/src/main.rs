// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]
#![no_main]

// rp235x-hal: Support for the RP235x Boot ROM's "Block" structures
pub mod block;

// We have to do this if we don't otherwise use it to ensure its vector table
// gets linked in.
use rp235x_pac as _;

use crate::block::ImageDef;
use riscv_rt::entry;

/// Tell the Boot ROM about our application
#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();

#[entry]
fn main() -> ! {
    // Default boot speed, until we bother raising it:
    const CYCLES_PER_MS: u32 = 8_000;

    unsafe { hubris_kern::startup::start_kernel(CYCLES_PER_MS) }
}
