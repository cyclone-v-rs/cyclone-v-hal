//! Intel/Altera Cyclone V SoC FPGA (dual Cortex-A9, `armv7a-none-eabi`).
//!
//! Owned by this module: physical addresses and capacities, the Boot ROM
//! handoff contract, and the physical memory linker script (`ld/memory.x`
//! beside this file). Not owned here: the boot arena policy (boot profile in
//! `sidermux-bootloader`) and installed DDR geometry (boards).
//!
//! # Boot ROM handoff
//!
//! Incoming convention (Boot ROM → image entry): `r0 = rom_info` address,
//! `r1 = rom_info` length. This is Cyclone V-specific; other SoCs may pass
//! different registers, supply no information, or require another firmware
//! stage. The runtime's outgoing `boot_main` convention is defined by
//! `sidermux-rt` and is independent of the incoming registers.

/// Base of the OCRAM window the Boot ROM loads the boot image into.
pub const BOOT_IMAGE_BASE: usize = 0xFFFF_0000;
/// Offset of the Boot ROM header slot within the boot image.
pub const BOOT_HEADER_OFFSET: usize = 0x40;
/// Offset of the handoff entry slot within the boot image.
pub const BOOT_ENTRY_OFFSET: usize = 0x4C;
/// Boot arena currently sized by the Cyclone V boot profile.
pub const BOOT_ARENA_SIZE: usize = 60 * 1024;

/// Base of HPS Watchdog Timer 0 (DesignWare-style register block).
const WDT0_BASE: usize = 0xFFD0_2000;
/// Offset of `WDT_CR` (control; bit 0 enables the watchdog).
const WDT0_CR: usize = 0x00;
/// Offset of `WDT_CRR` (counter restart; `0x76` services the counter).
const WDT0_CRR: usize = 0x0C;
const WDT_RESTART_MAGIC: u32 = 0x76;

/// Disable the watchdog left running by the Boot ROM.
///
/// The Cyclone V Boot ROM hands control over with the HPS watchdog running;
/// any lengthy initialization without servicing it resets the system. This
/// is intended to be the first HAL operation a bootloader invokes.
///
/// Sequence: service the counter (`WDT_CRR` restart magic), then clear the
/// enable bit in `WDT_CR`.
///
/// TODO(datasheet): confirm whether the Boot ROM engages the watchdog's
/// permutation lock (`WDT_PER` keying); this sequence assumes the
/// unlocked DesignWare-style register behavior.
pub fn watchdog_disable() {
    unsafe {
        let control = core::ptr::read_volatile((WDT0_BASE + WDT0_CR) as *const u32);
        core::ptr::write_volatile((WDT0_BASE + WDT0_CRR) as *mut u32, WDT_RESTART_MAGIC);
        core::ptr::write_volatile((WDT0_BASE + WDT0_CR) as *mut u32, control & !0x1);
    }
}
