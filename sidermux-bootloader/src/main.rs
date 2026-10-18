//! Boot ROM entry adapter and startup console for the selected board.
#![no_std]
#![no_main]

#[cfg(not(feature = "board-de1-soc"))]
compile_error!("sidermux-bootloader requires board-de1-soc");

#[cfg(feature = "board-de1-soc")]
mod app {
    use core::{fmt::Write as _, panic::PanicInfo, ptr};

    use sidermux_bsp::de1_soc::{Board, ConsoleError, InitError};

    #[derive(Debug, Clone, Copy)]
    #[expect(
        dead_code,
        reason = "Failure payloads are inspected through the debugger"
    )]
    pub enum BootError {
        Initialization(InitError),
        Uart(ConsoleError),
        ConsoleWrite,
        Panic,
        PeripheralsUnavailable,
    }

    core::arch::global_asm!(
        include_str!("profiles/cyclone-v/boot-entry.S"),
        runtime_start = sym sidermux_rt::_start,
    );

    /// Minimal debugger-visible state when console initialization fails.
    #[derive(Debug)]
    pub struct BootStatus {
        pub reset_cause: u32,
        pub error: Option<BootError>,
    }
    #[unsafe(no_mangle)]
    pub static mut BOOT_STATUS: BootStatus = BootStatus {
        reset_cause: 0,
        error: None,
    };

    #[unsafe(no_mangle)]
    pub extern "C" fn boot_main(_rom_info: usize, _rom_info_len: usize) -> ! {
        let Some(mut board) = Board::take() else {
            record_failure(BootError::PeripheralsUnavailable);
            loop {
                core::hint::spin_loop();
            }
        };
        let reset_cause = board.reset_cause();
        unsafe { ptr::write_volatile(ptr::addr_of_mut!(BOOT_STATUS.reset_cause), reset_cause) };
        let result = write_banner(&mut board, reset_cause);
        if let Err(error) = result {
            fail(&mut board, error);
        }
        idle(&mut board)
    }

    fn write_banner(board: &mut Board, reset_cause: u32) -> Result<(), BootError> {
        let console = board
            .initialize_console()
            .map_err(BootError::Initialization)?;
        if write!(
            console,
            "sidermux bootloader\r\nreset cause: {reset_cause:#010x}\r\n"
        )
        .is_err()
        {
            return Err(BootError::ConsoleWrite);
        }
        console.flush().map_err(BootError::Uart)
    }

    fn record_failure(error: BootError) {
        unsafe { ptr::write_volatile(ptr::addr_of_mut!(BOOT_STATUS.error), Some(error)) };
    }
    fn fail(board: &mut Board, error: BootError) -> ! {
        record_failure(error);
        idle(board)
    }
    fn idle(board: &mut Board) -> ! {
        loop {
            board.service_watchdogs();
            core::hint::spin_loop();
        }
    }
    #[panic_handler]
    fn panic(_info: &PanicInfo<'_>) -> ! {
        record_failure(BootError::Panic);
        // There is no owned Board here; never steal handles from a
        // driver that may have panicked while borrowing them. An inherited
        // watchdog may reset the system after this unexpected failure.
        loop {
            core::hint::spin_loop();
        }
    }
}
