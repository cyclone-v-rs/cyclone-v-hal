//! System manager: HPS I/O pin multiplexing.
//!
//! Per U-Boot `board/terasic/de1-soc/qts/pinmux_config.h` (sequential
//! `sys_mgr_init_table` applied from the pinmux group base) and the Cyclone V
//! HPS Technical Reference Manual pinmux chapter.
//!
//! The pinmux table is a sequence of 32-bit registers starting at
//! [`PINMUX_BASE`]; entry `i` lives at `PINMUX_BASE + 4 * i` and selects the
//! function of pin `i`: `0` keeps GPIO, `1` selects the dedicated function.
//! Entry order: EMACIO0-19, FLASHIO0-11, GENERALIO0-47, MIXED1IO0-20,
//! MIXED2IO0-7, then GPIO and USEFPGA selects.

/// System manager base address.
pub const SYSMGR_BASE: usize = 0xFFD0_8000;
/// Pinmux table base (system manager + 0x400).
///
/// The EMACIO/FLASHIO/GENERALIO/MIXED/GPIO-mux entries are sequential from
/// this base (entry `i` at `PINMUX_BASE + 4 * i`), which places UART0 RX/TX
/// at the vendor-documented `0xFFD084C4`/`0xFFD084C8`. The later USEFPGA
/// entries do NOT continue sequentially; this module does not touch them.
pub const PINMUX_BASE: usize = 0xFFD0_8400;

/// Table index of GENERALIO0 (32 entries precede it: EMACIO + FLASHIO).
pub const GENERALIO_TABLE_INDEX: usize = 32;
/// Table index of the UART0 RX pin (GENERALIO17), register `0xFFD084C4`.
pub const UART0_RX_TABLE_INDEX: usize = GENERALIO_TABLE_INDEX + 17;
/// Table index of the UART0 TX pin (GENERALIO18), register `0xFFD084C8`.
pub const UART0_TX_TABLE_INDEX: usize = GENERALIO_TABLE_INDEX + 18;

/// Pinmux value: dedicated (peripheral) function.
pub const PINMUX_DEDICATED_FUNCTION: u32 = 1;
