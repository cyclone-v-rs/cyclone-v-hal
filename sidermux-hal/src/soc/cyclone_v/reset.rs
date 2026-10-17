//! Reset manager operations: releasing peripherals from module reset.

use core::ptr;

use super::pac::{clock_manager, reset_manager};

/// Releases UART0 from peripheral module reset.
///
/// Ensures the L4 SP clock gate is on, then clears the UART0 bit in
/// `PERMODRST`. Peripheral module reset bits are level-sensitive: writing 0
/// deasserts the reset.
pub fn release_uart0() {
    // Clock gate for the L4 SP domain feeding UART0.
    let en = unsafe {
        ptr::read_volatile((clock_manager::CLKMGR_BASE + clock_manager::MAINPLL_EN) as *const u32)
    };
    unsafe {
        ptr::write_volatile(
            (clock_manager::CLKMGR_BASE + clock_manager::MAINPLL_EN) as *mut u32,
            en | clock_manager::MAINPLL_EN_L4SPCLK,
        );
    }

    let rst = unsafe {
        ptr::read_volatile((reset_manager::RSTMGR_BASE + reset_manager::PERMODRST) as *const u32)
    };
    unsafe {
        ptr::write_volatile(
            (reset_manager::RSTMGR_BASE + reset_manager::PERMODRST) as *mut u32,
            rst & !reset_manager::PERMODRST_UART0,
        );
    }
}
