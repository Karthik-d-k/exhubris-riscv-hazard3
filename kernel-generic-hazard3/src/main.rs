// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]
#![no_main]

#[cfg(feature = "kernel-blink")]
mod blink;

// rp235x-hal: Support for the RP235x Boot ROM's "Block" structures
// pub mod block;

// We have to do this if we don't otherwise use it to ensure its vector table
// gets linked in.
use rp235x_pac as _;

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

    p.RESETS.reset().modify(|_, w| w.io_bank0().clear_bit());
    while !p.RESETS.reset_done().read().io_bank0().bit() {}

    // Provide U-mode access for riscv and non-secure mode access for arm for all GPIOs
    // This may not work on pico 2(W) boards, refer: Errata `RP2350-E3`
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

    // This is used to prove that M-mode GPIO works fine, but U-mode doesn't due to Errata `RP2350-E3`
    #[cfg(feature = "kernel-blink")]
    {
        // All of this is executing in M-mode.
        blink::run_demo();
    }

    unsafe { hubris_kern::startup::start_kernel(cycles_per_ms) }
}
