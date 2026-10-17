//! Clock manager: main/peripheral/SDR PLL groups and the L4 clock sources.
//!
//! Register offsets and bitfields per U-Boot
//! `arch/arm/mach-socfpga/include/mach/clock_manager_gen5.h` (Cyclone V
//! HPS Technical Reference Manual).
//!
//! The UART reference clock is the L4 SP clock. Its derivation is documented
//! in [`crate::soc::cyclone_v::clocks`].

/// Clock manager base address.
pub const CLKMGR_BASE: usize = 0xFFD0_4000;

/// Main PLL group: VCO (source select, numerator, denominator, status).
pub const MAINPLL_VCO: usize = 0x40;
/// Main PLL group: main clock counter (the L3/L4 source ahead of dividers).
pub const MAINPLL_MAINCLK: usize = 0x4C;
/// Main PLL group: clock-gate enables (L4 SP is bit 1).
pub const MAINPLL_EN: usize = 0x60;
/// Main PLL group: L3 MP / L3 SP / L4 MP / L4 SP dividers.
pub const MAINPLL_MAINDIV: usize = 0x64;
/// Main PLL group: L4 MP and L4 SP source select (main or peripheral PLL).
pub const MAINPLL_L4SRC: usize = 0x70;
/// Peripheral PLL group: VCO.
pub const PERPLL_VCO: usize = 0x80;
/// Peripheral PLL group: `per_base_clk` counter (L4 source when selected).
pub const PERPLL_PERBASECLK: usize = 0x98;
/// Altera group: main clock counter (ahead of `MAINPLL_MAINCLK`).
pub const ALTR_MAINCLK: usize = 0xE4;

/// `MAINPLL_EN`: L4 SP clock gate.
pub const MAINPLL_EN_L4SPCLK: u32 = 1 << 1;

/// `MAINPLL_L4SRC`: L4 SP source select (0 = main PLL, 1 = peripheral PLL).
pub const L4SRC_L4SP_OFFSET: u32 = 1;
/// `MAINPLL_L4SRC`: L4 SP source is the main PLL.
pub const L4SRC_L4SP_MAIN: u32 = 0x0;
/// `MAINPLL_L4SRC`: L4 SP source is the peripheral PLL.
pub const L4SRC_L4SP_PERPLL: u32 = 0x1;

/// `MAINPLL_MAINDIV`: L4 SP divider field, power-of-two encoded.
pub const MAINDIV_L4SPCLK_OFFSET: u32 = 7;
/// `MAINPLL_MAINDIV`: L4 SP divider field mask.
pub const MAINDIV_L4SPCLK_MASK: u32 = 0x7;

/// VCO denominator field offset (`DENOM + 1` divides the source).
pub const VCO_DENOM_OFFSET: u32 = 16;
/// VCO denominator field mask.
pub const VCO_DENOM_MASK: u32 = 0x3F;
/// VCO numerator field offset (`NUMER + 1` multiplies the source).
pub const VCO_NUMER_OFFSET: u32 = 3;
/// VCO numerator field mask.
pub const VCO_NUMER_MASK: u32 = 0xFF;
/// VCO source select field offset.
pub const PERPLL_VCO_SSRC_OFFSET: u32 = 22;
/// VCO source select field mask.
pub const PERPLL_VCO_SSRC_MASK: u32 = 0x3;

/// Peripheral PLL VCO reference: onboard oscillator 1.
pub const SSRC_EOSC1: u32 = 0x0;
/// Peripheral PLL VCO reference: onboard oscillator 2.
pub const SSRC_EOSC2: u32 = 0x1;
/// Peripheral PLL VCO reference: FPGA-to-HPS periphery reference clock.
pub const SSRC_F2S: u32 = 0x2;
