//! Terasic DE1-SoC board data from one matching reference handoff.
//!
//! UART0 uses GENERALIO17/18 and the onboard USB-to-UART bridge, without
//! hardware flow control. See `de1_soc/HANDOFF.md` for source provenance.
//!
//! ```no_run
//! use sidermux_bsp::de1_soc::{Board, Console, InitError};
//! let mut board = Board::take().expect("board already acquired or not on core 0");
//! let reset_cause = board.reset_cause();
//! let _console = console(&mut board);
//! fn console(board: &mut Board) -> Result<&mut Console, InitError> {
//!     board.initialize_console()
//! }
//! ```

use sidermux_hal::soc::cyclone_v::{
    clocks,
    early_boot::BootSupport,
    pac::{ClockManager, Peripherals, ScanManager, SystemManager, Uart0},
    scanmgr,
    system_manager,
    uart::{self, Uart},
};

use crate::initialization::Initialization;

mod handoff;

pub use handoff::{CLOCKS, IOCSR, PINMUX};

/// HPS oscillator 1, specified by the DE1-SoC reference design.
pub const OSC1_HZ: u32 = 25_000_000;
/// Console baud rate for the onboard USB-to-UART bridge.
pub const CONSOLE_BAUD: u32 = 115_200;

/// The board's HPS UART0 console; the reusable driver belongs to HAL.
pub type Console = Uart<Uart0>;
/// Console transfer or configuration error.
pub type ConsoleError = uart::UartError;

/// Failure during the board's single initialization attempt.
#[derive(Debug, Clone, Copy)]
pub enum InitError {
    Io(system_manager::IoError),
    Clock(clocks::ClockError),
    Scan {
        chain: usize,
        error: scanmgr::ScanError,
    },
    UartReset,
    Uart(ConsoleError),
}

/// Owned early-boot resources for this board.
pub struct Board {
    clock_manager: ClockManager,
    system_manager: SystemManager,
    scan_manager: ScanManager,
    uart: Option<Uart0>,
    support: BootSupport,
    reset_cause: u32,
    console: Initialization<Console, InitError>,
}
impl Board {
    /// Acquire the board once, service inherited watchdogs, and capture reset causes.
    /// Must be called from privileged core 0, with core 1 parked (PAC contract).
    pub fn take() -> Option<Self> {
        let p = Peripherals::take()?;
        let mut support = BootSupport::new(p.reset_manager, p.timer0, p.watchdog0, p.watchdog1);
        support.service_watchdogs();
        let reset_cause = support.reset_cause();
        Some(Self {
            clock_manager: p.clock_manager,
            system_manager: p.system_manager,
            scan_manager: p.scan_manager,
            uart: Some(p.uart0),
            support,
            reset_cause,
            console: Initialization::Pending,
        })
    }
    /// Reset status captured before board initialization.
    pub fn reset_cause(&self) -> u32 {
        self.reset_cause
    }
    /// Initialize once; subsequent calls return the same console or cached failure.
    /// A failed hardware configuration requires a reset before another attempt.
    pub fn initialize_console(&mut self) -> Result<&mut Console, InitError> {
        self.console.get_or_init(|| {
            initialize_console(
                &mut self.clock_manager,
                &mut self.system_manager,
                &mut self.scan_manager,
                self.uart.take().expect("pending initialization owns UART0"),
                &mut self.support,
            )
        })
    }
    /// Keep inherited watchdogs serviced, including after initialization failure.
    pub fn service_watchdogs(&mut self) {
        self.support.service_watchdogs();
    }
}

/// Follow the vendor freeze → clocks/IOCSR/pinmux → reset release → thaw flow.
/// Leave warm-reset I/O restoration enabled if programming fails midway.
fn initialize_console(
    clock_manager: &mut ClockManager,
    system_manager: &mut SystemManager,
    scan_manager: &mut ScanManager,
    uart: Uart0,
    support: &mut BootSupport,
) -> Result<Uart<Uart0>, InitError> {
    support.service_watchdogs();
    support.init_timer();
    support
        .delay_us(1, OSC1_HZ)
        .map_err(|error| InitError::Io(system_manager::IoError::Timer(error)))?;
    system_manager::freeze(system_manager, support, OSC1_HZ).map_err(InitError::Io)?;
    support.prepare_io();
    clocks::configure(clock_manager, support, &CLOCKS, OSC1_HZ).map_err(InitError::Clock)?;
    let clocks = clocks::early_clocks(clock_manager, OSC1_HZ).map_err(InitError::Clock)?;

    system_manager::warm_reset_io(system_manager, true);
    for (chain, config) in IOCSR.iter().enumerate() {
        if chain == 3 {
            system_manager::prepare_hio_scan(system_manager);
        }
        scanmgr::configure(scan_manager, support, chain, config)
            .map_err(|error| InitError::Scan { chain, error })?;
    }
    system_manager::warm_reset_io(system_manager, false);

    system_manager::warm_reset_io(system_manager, true);
    system_manager::configure_pinmux(system_manager, PINMUX).map_err(InitError::Io)?;
    system_manager::warm_reset_io(system_manager, false);

    support.release_uart0(clock_manager);
    if !support.uart0_released() {
        return Err(InitError::UartReset);
    }
    system_manager::thaw(system_manager, support, OSC1_HZ).map_err(InitError::Io)?;
    Uart::new(uart, clocks.uart0_hz, CONSOLE_BAUD).map_err(InitError::Uart)
}
