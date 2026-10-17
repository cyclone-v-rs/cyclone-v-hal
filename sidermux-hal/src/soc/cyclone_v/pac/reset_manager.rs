//! Reset manager: peripheral module resets.
//!
//! Register offsets and bitfields per U-Boot
//! `arch/arm/mach-socfpga/include/mach/reset_manager_gen5.h` (Cyclone V
//! HPS Technical Reference Manual). Note the bank naming: on gen5 the
//! peripheral bank containing UART is `permodrst` (offset 0x14), not the
//! `per1modrst` name used by Arria 10.

/// Reset manager base address.
pub const RSTMGR_BASE: usize = 0xFFD0_5000;

/// Peripheral module reset bank 1 (EMAC, NAND, QSPI, L4WD, timers, UART,
/// SPIM, SDMMC, DMA, SDR).
pub const PERMODRST: usize = 0x14;

/// `PERMODRST`: UART0 module reset (1 = held in reset).
pub const PERMODRST_UART0: u32 = 1 << 16;
