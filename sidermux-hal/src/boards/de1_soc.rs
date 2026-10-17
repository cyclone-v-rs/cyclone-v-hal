//! Terasic DE1-SoC board: pin routing and board clock facts.
//!
//! Console: HPS UART0, routed to the on-board USB-to-UART bridge (the micro
//! USB connector separate from USB-Blaster). The board reference
//! configuration keeps UART0 on its dedicated HPS pins with no flow
//! control; the pinmux entries below mirror that selection from U-Boot's
//! `board/terasic/de1-soc/qts/pinmux_config.h` (GENERALIO17/18 select the
//! dedicated function, `UART0USEFPGA` stays at its reset default of 0).

use crate::soc::cyclone_v::{
    clocks::{self, ClockError, EarlyClocks},
    pac::system_manager,
    pinmux,
};

/// Frequency of the board's HPS oscillator 1 (DE1-SoC User Manual).
pub const OSC1_HZ: u32 = 25_000_000;

/// Default console baud rate for the DE1-SoC USB-to-UART bridge.
pub const CONSOLE_BAUD: u32 = 115_200;

/// Proof that the UART0 pins are routed to their dedicated HPS function.
///
/// Produced only by [`configure_uart0_pins`]; the UART driver consumes it,
/// so a UART cannot be constructed on unrouted pins.
#[derive(Debug)]
pub struct Uart0Pins {
    _priv: (),
}

/// Derives early clock facts (the UART reference frequency) from the board
/// oscillator and the current clock manager configuration.
pub fn early_clocks() -> Result<EarlyClocks, ClockError> {
    clocks::early_clocks(OSC1_HZ)
}

/// Routes UART0 RX/TX to their dedicated HPS function (no flow control).
pub fn configure_uart0_pins() -> Uart0Pins {
    let dedicated = system_manager::PINMUX_DEDICATED_FUNCTION;
    // Errors are impossible for the constant 1; a failure would mean the
    // mechanism constants drifted apart.
    pinmux::set_entry(system_manager::UART0_RX_TABLE_INDEX, dedicated)
        .expect("UART0 RX pinmux entry is valid");
    pinmux::set_entry(system_manager::UART0_TX_TABLE_INDEX, dedicated)
        .expect("UART0 TX pinmux entry is valid");
    Uart0Pins { _priv: () }
}
