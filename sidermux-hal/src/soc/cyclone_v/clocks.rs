//! Early clock facts derived from the current clock manager configuration,
//! with the vendor Gen5 PLL programming sequence.
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
//!
//! [`configure`] programs the board handoff from OCRAM with I/O frozen.
//! Frequency results describe the registers; they are not measurements.

use sidermux_pac::prelude::*;

use super::{
    early_boot::BootSupport,
    pac::{ClockManager, clock_manager as pac},
};

/// Clock facts needed by early peripherals (currently the UART reference).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EarlyClocks {
    /// Register-derived L4 SP clock in Hz: the UART0/UART1 reference frequency.
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
    /// The clock manager did not reach the expected state (lock or busy
    /// clear) within the polling budget.
    Timeout,
}

/// Named clock counter slots; configuration data cannot select arbitrary MMIO.
#[derive(Debug, Clone, Copy)]
pub enum ClockCounter {
    Mpu,
    AlteraMpu,
    AlteraMain,
    AlteraDebugAt,
    Main,
    DebugAt,
    MainQspi,
    MainNandSdmmc,
    CfgUser0,
    Emac0,
    Emac1,
    PerQspi,
    PerNandSdmmc,
    PerBase,
    User1,
}
impl ClockCounter {
    fn write(self, manager: &ClockManager, value: u32) {
        match self {
            Self::Mpu => manager.mpu.set(value),
            Self::AlteraMpu => manager.altera_mpu.set(value),
            Self::AlteraMain => manager.altera_main.set(value),
            Self::AlteraDebugAt => manager.altera_debug_at.set(value),
            Self::Main => manager.main.set(value),
            Self::DebugAt => manager.debug_at.set(value),
            Self::MainQspi => manager.main_qspi.set(value),
            Self::MainNandSdmmc => manager.main_nandsdmmc.set(value),
            Self::CfgUser0 => manager.cfg_user0.set(value),
            Self::Emac0 => manager.emac0.set(value),
            Self::Emac1 => manager.emac1.set(value),
            Self::PerQspi => manager.per_qspi.set(value),
            Self::PerNandSdmmc => manager.per_nandsdmmc.set(value),
            Self::PerBase => manager.per_base.set(value),
            Self::User1 => manager.user1.set(value),
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub enum ClockDivider {
    Main,
    Debug,
    Trace,
    Peripheral,
    Gpio,
}
impl ClockDivider {
    fn write(self, manager: &ClockManager, value: u32) {
        match self {
            Self::Main => manager.main_divider.set(value),
            Self::Debug => manager.debug_divider.set(value),
            Self::Trace => manager.trace_divider.set(value),
            Self::Peripheral => manager.per_divider.set(value),
            Self::Gpio => manager.gpio_divider.set(value),
        }
    }
}
/// Board-generated register values, all from a single QTS handoff.
pub struct ClockConfig {
    pub vco: [u32; 3],
    pub counters: &'static [(ClockCounter, u32)],
    pub dividers: &'static [(ClockDivider, u32)],
    pub sdram: [u32; 4],
    pub sources: [u32; 2],
}
fn wait(
    mut read: impl FnMut() -> u32,
    mask: u32,
    expected: u32,
    support: &mut BootSupport,
) -> Result<(), ClockError> {
    for i in 0..1_000_000 {
        if read() & mask == expected {
            return Ok(());
        }
        if i % 256 == 0 {
            support.service_watchdogs();
        }
    }
    Err(ClockError::Timeout)
}
/// Vendor Gen5 sequence. Run from OCRAM with I/O frozen and resets established.
pub fn configure(
    manager: &mut ClockManager,
    support: &mut BootSupport,
    cfg: &ClockConfig,
    osc: u32,
) -> Result<(), ClockError> {
    manager
        .per_enable
        .set(manager.per_enable.get() & !pac::PERPLL_EN_NAND);
    manager.main_enable.set(pac::MAINPLL_EN_EARLY);
    manager.sdr_enable.set(0);
    manager.per_enable.set(0);
    manager.bypass.set(pac::BYPASS_ALL);
    wait(|| manager.stat.get(), pac::STAT_BUSY, 0, support)?;
    let vcos = [&manager.main_vco, &manager.per_vco, &manager.sdr_vco];
    for register in vcos {
        register.set(pac::VCO_RESET);
    }
    manager.per_source.set(pac::PERPLL_SRC_RESET);
    manager.l4_source.set(0);
    support.delay_us(5, osc).map_err(|_| ClockError::Timeout)?;
    for (register, value) in vcos.into_iter().zip(cfg.vco) {
        register.set(value);
    }
    support.delay_us(7, osc).map_err(|_| ClockError::Timeout)?;
    for &(counter, value) in cfg.counters {
        counter.write(manager, value);
    }
    for (register, value) in vcos.into_iter().zip(cfg.vco) {
        register.set(value | pac::MAINPLL_VCO_EN);
    }
    for &(divider, value) in cfg.dividers {
        divider.write(manager, value);
    }
    let mut stable = 0;
    for i in 0..1_000_000 {
        if manager.inter.get() & pac::INTER_ALL_LOCKED == pac::INTER_ALL_LOCKED {
            stable += 1;
        } else {
            stable = 0;
        }
        if stable == 10 {
            break;
        }
        if i % 256 == 0 {
            support.service_watchdogs();
        }
    }
    if stable != 10 {
        return Err(ClockError::Timeout);
    }
    let sdram = [
        &manager.ddr_dqs,
        &manager.ddr_2xdqs,
        &manager.ddr_dq,
        &manager.user2,
    ];
    for (register, value) in sdram.into_iter().zip(cfg.sdram) {
        register.set(value & pac::SDRPLL_COUNTER_MASK);
    }
    for register in vcos {
        register.set(register.get() | pac::VCO_OUTRESETALL);
    }
    for register in vcos {
        register.set(register.get() & !pac::VCO_OUTRESETALL);
    }
    for (register, value) in sdram.into_iter().zip(cfg.sdram) {
        wait(|| register.get(), pac::SDRPLL_PHASE_MASK, 0, support)?;
        register.set(value);
        wait(|| register.get(), pac::SDRPLL_PHASE_MASK, 0, support)?;
    }
    manager.bypass.set(0);
    wait(|| manager.stat.get(), pac::STAT_BUSY, 0, support)?;
    manager.ctrl.set(manager.ctrl.get() | pac::CTRL_SAFEMODE);
    wait(|| manager.stat.get(), pac::STAT_BUSY, 0, support)?;
    manager.per_source.set(cfg.sources[0]);
    manager.l4_source.set(cfg.sources[1]);
    manager.main_enable.set(u32::MAX);
    manager.per_enable.set(u32::MAX);
    manager.sdr_enable.set(u32::MAX);
    manager.inter.set(pac::INTER_LOST_LOCK);
    Ok(())
}

/// Derives early clock facts from the current clock manager state for a
/// board whose primary oscillator runs at `osc1_hz`.
pub fn early_clocks(manager: &ClockManager, osc1_hz: u32) -> Result<EarlyClocks, ClockError> {
    let registers = ClockRegisters {
        l4src: manager.l4_source.get(),
        main_vco: manager.main_vco.get(),
        altr_mainclk: manager.altera_main.get(),
        mainclk: manager.main.get(),
        maindiv: manager.main_divider.get(),
        per_vco: manager.per_vco.get(),
        perbaseclk: manager.per_base.get(),
    };
    derive(osc1_hz, &registers).map(|uart0_hz| EarlyClocks { uart0_hz })
}

struct ClockRegisters {
    l4src: u32,
    main_vco: u32,
    altr_mainclk: u32,
    mainclk: u32,
    maindiv: u32,
    per_vco: u32,
    perbaseclk: u32,
}

fn derive(osc1: u32, r: &ClockRegisters) -> Result<u32, ClockError> {
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

    const OSC1: u32 = 25_000_000;

    #[test]
    fn vendor_typical_main_source_yields_100mhz() {
        // 25 MHz * 64 (numer 63, denom 0) = 1.6 GHz VCO; main clock counter
        // 15 -> 100 MHz; Altera counter 0; L4 SP divider 0.
        let r = ClockRegisters {
            l4src: 0,
            main_vco: (63 << pac::VCO_NUMER_OFFSET),
            altr_mainclk: 0,
            mainclk: 15,
            maindiv: 0,
            per_vco: 0,
            perbaseclk: 0,
        };
        assert_eq!(derive(OSC1, &r), Ok(100_000_000));
    }

    #[test]
    fn perpll_source_yields_configured_rate() {
        // Per VCO from EOSC1: 25 MHz * 40 = 1 GHz; per_base counter 9 ->
        // 100 MHz; L4 SP divider 1 -> 50 MHz.
        let r = ClockRegisters {
            l4src: 1 << pac::L4SRC_L4SP_OFFSET,
            main_vco: 0,
            altr_mainclk: 0,
            mainclk: 0,
            maindiv: 1 << pac::MAINDIV_L4SPCLK_OFFSET,
            per_vco: pac::SSRC_EOSC1 << pac::PERPLL_VCO_SSRC_OFFSET | (39 << pac::VCO_NUMER_OFFSET),
            perbaseclk: 9,
        };
        assert_eq!(derive(OSC1, &r), Ok(50_000_000));
    }

    #[test]
    fn f2s_reference_without_board_data_is_unsupported() {
        let r = ClockRegisters {
            l4src: 1 << pac::L4SRC_L4SP_OFFSET,
            main_vco: 0,
            altr_mainclk: 0,
            mainclk: 0,
            maindiv: 0,
            per_vco: pac::SSRC_F2S << pac::PERPLL_VCO_SSRC_OFFSET,
            perbaseclk: 0,
        };
        assert_eq!(derive(OSC1, &r), Err(ClockError::UnsupportedSource));
    }
}
