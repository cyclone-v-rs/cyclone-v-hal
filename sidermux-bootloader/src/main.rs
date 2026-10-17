//! Bootable image for sidermux targets.
//!
//! Layering: architecture startup (`sidermux-rt`) → SoC handoff adapter
//! (this crate's `boot_main`) → common boot logic. `boot_main`'s signature
//! mirrors the Cyclone V Boot ROM handoff; it is the adapter, not a portable
//! ABI.
//!
//! The runtime startup masks IRQ/FIQ, selects Supervisor mode, sets the SVC
//! stack, clears high-vector selection, and installs the image's vector
//! table in VBAR before entering Rust. Installing the table only decides
//! where exceptions enter; the interrupt controller, handler context, and
//! exception-mode stacks are not initialized, so interrupts stay masked.
#![no_std]
#![no_main]

use core::panic::PanicInfo;

use sidermux_hal::soc::cyclone_v;

// Binaries must select exactly one SoC. With a single supported SoC the
// missing-selection arm below suffices; when a second SoC lands, add the
// conflicting-pair arm (all(feature = "a", feature = "b")).
#[cfg(not(feature = "cyclone-v"))]
compile_error!("sidermux-bootloader requires exactly one SoC feature; supported: cyclone-v");

// Cyclone V boot profile: vector table at the image base, Boot ROM header
// slot at +0x40, handoff entry slot at +0x4C.
#[cfg(feature = "cyclone-v")]
core::arch::global_asm!(
    include_str!("profiles/cyclone-v/boot-entry.S"),
    runtime_start = sym sidermux_rt::_start,
);

/// Cyclone V Boot ROM handoff adapter.
///
/// Receives the SoC-specific incoming registers (`r0 = rom_info` address,
/// `r1 = rom_info` length) and will hand normalized boot information to the
/// common boot logic, so later SoC adapters can feed the same flow.
#[unsafe(no_mangle)]
pub extern "C" fn boot_main(_rom_info: usize, _rom_info_len: usize) -> ! {
    // First HAL operation: the Boot ROM leaves the watchdog running; any
    // lengthy initialization resets the system without disabling it.
    cyclone_v::watchdog_disable();

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
