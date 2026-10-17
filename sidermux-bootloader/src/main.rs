//! Bootable image for sidermux targets.
//!
//! Layering: architecture startup (`sidermux-rt`) → SoC handoff adapter
//! (this crate's `boot_main`) → common boot logic. `boot_main`'s signature
//! mirrors the Cyclone V Boot ROM handoff; it is the adapter, not a portable
//! ABI.
//!
//! The runtime startup masks IRQ/FIQ, selects Supervisor mode, sets the SVC
//! stack, clears high-vector selection, and installs the image's vector
//! table in VBAR before entering Rust. Installing the table only decides
//! where exceptions enter; the interrupt controller, handler context, and
//! exception-mode stacks are not initialized, so interrupts stay masked.
//!
//! Bring-up order in `boot_main`: disable the inherited watchdog first
//! (UART debugging becomes periodic resets otherwise), then clocks, pins,
//! reset release, and UART configuration. The current milestone prints a
//! repeated banner; RX polling follows once TX is verified on hardware.
#![no_std]
#![no_main]

#[cfg(not(feature = "board-de1-soc"))]
compile_error!("sidermux-bootloader requires exactly one board feature; supported: board-de1-soc");

#[cfg(feature = "board-de1-soc")]
mod app {
    use core::{fmt::Write as _, panic::PanicInfo};

    use sidermux_hal::{
        boards::de1_soc,
        soc::cyclone_v::{
            self,
            reset,
            uart::{self, Uart},
        },
    };

    // Cyclone V boot profile: vector table at the image base, Boot ROM
    // header slot at +0x40, handoff entry slot at +0x4C.
    core::arch::global_asm!(
        include_str!("profiles/cyclone-v/boot-entry.S"),
        runtime_start = sym sidermux_rt::_start,
    );

    /// Console baud rate for the DE1-SoC USB-to-UART bridge.
    const CONSOLE_BAUD: u32 = de1_soc::CONSOLE_BAUD;

    /// Busy-wait length between banner repeats. A coarse cadence only; a
    /// timer-based delay replaces this once a counter is available.
    const REPEAT_DELAY_ITERATIONS: u32 = 25_000_000;

    /// Cyclone V Boot ROM handoff adapter.
    ///
    /// Receives the SoC-specific incoming registers (`r0 = rom_info`
    /// address, `r1 = rom_info` length) and will hand normalized boot
    /// information to the common boot logic, so later SoC adapters can feed
    /// the same flow.
    #[unsafe(no_mangle)]
    pub extern "C" fn boot_main(_rom_info: usize, _rom_info_len: usize) -> ! {
        // First HAL operation: the Boot ROM leaves the watchdog running; any
        // lengthy initialization resets the system without disabling it.
        cyclone_v::watchdog_disable();

        // Console bring-up: derive the UART reference from the clock
        // manager, route the board pins, release the module, configure.
        let clocks = match de1_soc::early_clocks() {
            Ok(clocks) => clocks,
            Err(_) => fatal(),
        };
        let pins = de1_soc::configure_uart0_pins();
        reset::release_uart0();
        let mut uart = match Uart::new(uart::UART0, pins, clocks.uart0_hz, CONSOLE_BAUD) {
            Ok(uart) => uart,
            Err(_) => fatal(),
        };

        // Milestone: repeated TX banner. IRQ/FIQ are still masked; the only
        // output path is polled transmit.
        let mut repeats: u32 = 0;
        loop {
            let _ = write!(
                uart,
                "\r\nsidermux: Rust entry reached (repeats={repeats})\r\n"
            );
            let _ = uart.flush();
            repeats = repeats.wrapping_add(1);
            spin_delay(REPEAT_DELAY_ITERATIONS);
        }
    }

    /// Busy-wait loop of roughly `iterations` CPU iterations. Not a time
    /// guarantee; the loop body is kept opaque so it is not optimized away.
    fn spin_delay(iterations: u32) {
        for i in 0..iterations {
            core::hint::black_box(i);
        }
    }

    /// Deterministic stop for bring-up failures that cannot be reported
    /// (the console itself failed to come up).
    #[inline(never)]
    fn fatal() -> ! {
        loop {
            core::hint::spin_loop();
        }
    }

    #[panic_handler]
    fn panic(_info: &PanicInfo<'_>) -> ! {
        loop {
            core::hint::spin_loop();
        }
    }
}

#[cfg(not(feature = "board-de1-soc"))]
fn main() {}
