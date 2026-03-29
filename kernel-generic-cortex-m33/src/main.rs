// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]
#![no_main]

// We have to do this if we don't otherwise use it to ensure its vector table
// gets linked in.
use rp235x_pac::{self as _, XOSC};

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

// Initialize the XOSC to run @ 12 MHz
fn xosc_init(xosc: &XOSC) {
    // set the frequency range to 1-15 MHz
    xosc.ctrl().write(|w| {
        w.freq_range()
            .variant(rp235x_pac::xosc::ctrl::FREQ_RANGE_A::_1_15MHZ)
    });

    // set xosc startup delay (1ms is sufficient for 12 MHz), refer RP2350 Datasheet: 8.2.4
    xosc.startup().write(|w| unsafe { w.delay().bits(47) });

    // set the enable bit now that we have set freq range and startup delay
    xosc.ctrl()
        .write(|w| w.enable().variant(rp235x_pac::xosc::ctrl::ENABLE_A::ENABLE));

    // Wait for XOSC to be stable
    while !xosc.status().read().stable().bit_is_set() {}
}

// Initialize the PLL to run the system clock at 150 MHz, with a 12 MHz reference from XOSC.
fn pll_init(resets: &rp235x_pac::RESETS, pll_sys: &rp235x_pac::PLL_SYS) {
    // > pico-sdk-2.2.0\src\rp2_common\hardware_clocks\scripts> python .\vcocalc.py 150
    // Requested: 150.0 MHz
    // Achieved:  150.0 MHz
    // REFDIV:    1
    // FBDIV:     125 (VCO = 1500.0 MHz)
    // PD1:       5
    // PD2:       2

    // 1. Reset PLL_SYS
    resets.reset().modify(|_, w| w.pll_sys().set_bit());
    resets.reset().modify(|_, w| w.pll_sys().clear_bit());
    while !resets.reset_done().read().pll_sys().bit() {}

    // 2. Configure PLL: REFDIV=1, FBDIV=125
    pll_sys.cs().write(|w| unsafe { w.refdiv().bits(1) });
    pll_sys
        .fbdiv_int()
        .write(|w| unsafe { w.fbdiv_int().bits(125) });

    // 3. Power on PLL (clear PD and VCOPD)
    pll_sys
        .pwr()
        .modify(|_, w| w.pd().clear_bit().vcopd().clear_bit());

    // 4. Wait for PLL to lock
    while !pll_sys.cs().read().lock().bit() {}

    // 5. Configure post dividers: POSTDIV1=5, POSTDIV2=2
    pll_sys.prim().write(|w| unsafe {
        w.postdiv1().bits(5);
        w.postdiv2().bits(2)
    });

    // 6. Power on post divider
    pll_sys.pwr().modify(|_, w| w.postdivpd().clear_bit());
}

// Initialize clk_sys to run at 150 MHz from PLL_SYS, and route XOSC to clk_peri for UART use.
fn clock_init(clocks: &rp235x_pac::CLOCKS) {
    // Switch clk_sys to PLL via aux mux
    // First set auxsrc to PLL_SYS (0) while src is still clk_ref
    clocks
        .clk_sys_ctrl()
        .modify(|_, w| w.auxsrc().clksrc_pll_sys());
    // Then glitchlessly switch src to aux
    clocks
        .clk_sys_ctrl()
        .modify(|_, w| w.src().clksrc_clk_sys_aux());

    // Route XOSC (12 MHz) to clk_peri (for UART)
    clocks.clk_peri_ctrl().write(|w| {
        unsafe { w.auxsrc().bits(4) }; // xosc_clksrc
        w.enable().set_bit();
        w
    });
}

#[entry]
fn main() -> ! {
    let p = unsafe { rp235x_pac::Peripherals::steal() };

    // Initialize XOSC
    xosc_init(&p.XOSC);

    // Initialize PLL
    pll_init(&p.RESETS, &p.PLL_SYS);

    // switch system clock to PLL output and xosc to UART peripheral clock
    clock_init(&p.CLOCKS);

    // clk_sys = 150 MHz
    let cycles_per_ms = 150_000;

    unsafe { hubris_kern::startup::start_kernel(cycles_per_ms) }
}
