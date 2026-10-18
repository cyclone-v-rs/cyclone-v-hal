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

pub mod clocks;
pub use sidermux_pac::cyclone_v as pac;
pub mod early_boot;
pub mod reset;
pub mod scanmgr;
pub mod system_manager;
pub mod timer;
pub mod uart;
pub mod watchdog;

/// Base of the OCRAM window the Boot ROM loads the boot image into.
pub const BOOT_IMAGE_BASE: usize = 0xFFFF_0000;
/// Offset of the Boot ROM header slot within the boot image.
pub const BOOT_HEADER_OFFSET: usize = 0x40;
/// Offset of the handoff entry slot within the boot image.
pub const BOOT_ENTRY_OFFSET: usize = 0x4C;
/// Boot arena currently sized by the Cyclone V boot profile.
pub const BOOT_ARENA_SIZE: usize = 60 * 1024;
