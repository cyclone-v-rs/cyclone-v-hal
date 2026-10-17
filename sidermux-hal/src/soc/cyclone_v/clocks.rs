//! Early clock facts derived from the current clock manager configuration.
//!
//! The UART reference clock is the L4 SP clock. This module derives it from
//! the clock manager registers using the same path as U-Boot's
//! `cm_get_l4_sp_clk_hz()` (arch/arm/mach-socfpga/clock_manager_gen5.c):
//!
//! ```text
//! L4 SP source selected by MAINPLL_L4SRC[1]:
//!   main PLL  -> main VCO / (ALTR_MAINCLK+1) / (MAINCLK+1)
//!   per PLL   -> per VCO  / (PERBASECLK+1)
//! then -> / 2^MAINDIV_L4SPCLK
//! ```
//!
//! The board oscillator frequency is supplied by the caller (board data);
//! the rest is SoC logic. No frequency is hardcoded.

use core::ptr;

use super::pac::clock_manager as pac;

/// Clock facts needed by early peripherals (currently the UART reference).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EarlyClocks {
    /// L4 SP clock in Hz: the UART0/UART1 reference frequency.
    pub uart0_hz: u32,
}

/// Clock derivation errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockError {
    /// The peripheral PLL references a clock whose frequency is board data
    /// this build does not carry (EOSC2 or the FPGA-to-HPS reference).
    UnsupportedSource,
    /// Derived clock or a divider is outside the range the clock tree
    /// defines (a misconfigured clock manager).
    BadConfiguration,
}

/// Derives early clock facts from the current clock manager state for a
/// board whose primary oscillator runs at `osc1_hz`.
pub fn early_clocks(osc1_hz: u32) -> Result<EarlyClocks, ClockError> {
    let reg = |offset: usize| -> u32 {
        unsafe { ptr::read_volatile((pac::CLKMGR_BASE + offset) as *const u32) }
    };

    let l4src = reg(pac::MAINPLL_L4SRC);
    let source = (l4src >> pac::L4SRC_L4SP_OFFSET) & 1;

    let pre_div_clock = if source == pac::L4SRC_L4SP_MAIN {
        let main_vco = vco_hz(osc1_hz, reg(pac::MAINPLL_VCO))?;
        // Two sequential counters, same as the U-Boot derivation order.
        let clock = main_vco / (reg(pac::ALTR_MAINCLK) + 1);
        clock / (reg(pac::MAINPLL_MAINCLK) + 1)
    } else if source == pac::L4SRC_L4SP_PERPLL {
        let per_vco = per_vco_hz(osc1_hz, reg(pac::PERPLL_VCO))?;
        per_vco / (reg(pac::PERPLL_PERBASECLK) + 1)
    } else {
        return Err(ClockError::BadConfiguration);
    };

    let l4sp_div =
        (reg(pac::MAINPLL_MAINDIV) >> pac::MAINDIV_L4SPCLK_OFFSET) & pac::MAINDIV_L4SPCLK_MASK;
    let uart0_hz = pre_div_clock >> l4sp_div;

    if uart0_hz == 0 {
        return Err(ClockError::BadConfiguration);
    }
    Ok(EarlyClocks { uart0_hz })
}

/// VCO output: source / (denom + 1) * (numer + 1), computed in `u64` so the
/// multiply cannot wrap before the range check.
fn vco_hz(source_hz: u32, vco: u32) -> Result<u32, ClockError> {
    let denom = ((vco >> pac::VCO_DENOM_OFFSET) & pac::VCO_DENOM_MASK) as u64 + 1;
    let numer = ((vco >> pac::VCO_NUMER_OFFSET) & pac::VCO_NUMER_MASK) as u64 + 1;
    let hz = u64::from(source_hz) / denom * numer;
    u32::try_from(hz).map_err(|_| ClockError::BadConfiguration)
}

/// Peripheral PLL VCO output, resolving its reference source. EOSC2 and the
/// FPGA reference are board/FPGA data; a build that does not carry them
/// reports [`ClockError::UnsupportedSource`].
fn per_vco_hz(osc1_hz: u32, vco: u32) -> Result<u32, ClockError> {
    let source = (vco >> pac::PERPLL_VCO_SSRC_OFFSET) & pac::PERPLL_VCO_SSRC_MASK;
    match source {
        pac::SSRC_EOSC1 => vco_hz(osc1_hz, vco),
        pac::SSRC_EOSC2 | pac::SSRC_F2S => Err(ClockError::UnsupportedSource),
        _ => Err(ClockError::BadConfiguration),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Register snapshot for a pure reimplementation of the derivation.
    struct Regs {
        l4src: u32,
        main_vco: u32,
        altr_mainclk: u32,
        mainclk: u32,
        maindiv: u32,
        per_vco: u32,
        perbaseclk: u32,
    }

    fn derive_pure(osc1: u32, r: &Regs) -> Result<u32, ClockError> {
        let source = (r.l4src >> pac::L4SRC_L4SP_OFFSET) & 1;
        let pre_div = if source == pac::L4SRC_L4SP_MAIN {
            let clock = vco_hz(osc1, r.main_vco)?;
            let clock = clock / (r.altr_mainclk + 1);
            clock / (r.mainclk + 1)
        } else if source == pac::L4SRC_L4SP_PERPLL {
            per_vco_hz(osc1, r.per_vco)? / (r.perbaseclk + 1)
        } else {
            return Err(ClockError::BadConfiguration);
        };
        let l4sp_div = (r.maindiv >> pac::MAINDIV_L4SPCLK_OFFSET) & pac::MAINDIV_L4SPCLK_MASK;
        let clock = pre_div >> l4sp_div;
        if clock == 0 {
            return Err(ClockError::BadConfiguration);
        }
        Ok(clock)
    }

    const OSC1: u32 = 25_000_000;

    #[test]
    fn vendor_typical_main_source_yields_100mhz() {
        // 25 MHz * 64 (numer 63, denom 0) = 1.6 GHz VCO; main clock counter
        // 15 -> 100 MHz; Altera counter 0; L4 SP divider 0.
        let r = Regs {
            l4src: 0,
            main_vco: (63 << pac::VCO_NUMER_OFFSET),
            altr_mainclk: 0,
            mainclk: 15,
            maindiv: 0,
            per_vco: 0,
            perbaseclk: 0,
        };
        assert_eq!(derive_pure(OSC1, &r), Ok(100_000_000));
    }

    #[test]
    fn perpll_source_yields_configured_rate() {
        // Per VCO from EOSC1: 25 MHz * 40 = 1 GHz; per_base counter 9 ->
        // 100 MHz; L4 SP divider 1 -> 50 MHz.
        let r = Regs {
            l4src: 1 << pac::L4SRC_L4SP_OFFSET,
            main_vco: 0,
            altr_mainclk: 0,
            mainclk: 0,
            maindiv: 1 << pac::MAINDIV_L4SPCLK_OFFSET,
            per_vco: pac::SSRC_EOSC1 << pac::PERPLL_VCO_SSRC_OFFSET | (39 << pac::VCO_NUMER_OFFSET),
            perbaseclk: 9,
        };
        assert_eq!(derive_pure(OSC1, &r), Ok(50_000_000));
    }

    #[test]
    fn f2s_reference_without_board_data_is_unsupported() {
        let r = Regs {
            l4src: 1 << pac::L4SRC_L4SP_OFFSET,
            main_vco: 0,
            altr_mainclk: 0,
            mainclk: 0,
            maindiv: 0,
            per_vco: pac::SSRC_F2S << pac::PERPLL_VCO_SSRC_OFFSET,
            perbaseclk: 0,
        };
        assert_eq!(derive_pure(OSC1, &r), Err(ClockError::UnsupportedSource));
    }
}
