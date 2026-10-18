//! Reset-manager operations on an exclusively owned register block.
use sidermux_pac::prelude::*;

use super::pac::{ClockManager, ResetManager, clock_manager, reset_manager};

pub fn prepare_io(reset: &mut ResetManager) {
    let keep = reset_manager::PERMODRST_WATCHDOG0
        | reset_manager::PERMODRST_WATCHDOG1
        | reset_manager::PERMODRST_OSC1TIMER0;
    reset
        .peripheral
        .set((reset.peripheral.get() & keep) | !keep);
    reset.peripheral2.set(u32::MAX);
    let old = reset.misc.get();
    reset.misc.set(old | reset_manager::MISCMODRST_SCANMGR);
    reset.misc.set(old & !reset_manager::MISCMODRST_SCANMGR);
}
pub fn cause(reset: &ResetManager) -> u32 {
    reset.stat.get()
}
pub fn peripheral_state(reset: &ResetManager) -> u32 {
    reset.peripheral.get()
}
pub fn uart0_released(reset: &ResetManager) -> bool {
    !reset
        .peripheral
        .is_set(reset_manager::PeripheralReset::UART0)
}
pub fn release_uart0(reset: &mut ResetManager, clocks: &mut ClockManager) {
    clocks
        .main_enable
        .modify(clock_manager::MainEnable::L4SP::SET);
    reset
        .peripheral
        .modify(reset_manager::PeripheralReset::UART0::CLEAR);
    let _ = reset.peripheral.get();
}
