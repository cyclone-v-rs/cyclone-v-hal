//! Service inherited watchdogs without assuming their enable can be cleared.
use sidermux_pac::prelude::*;

use super::pac::{ResetManager, Watchdog0, Watchdog1, reset_manager, watchdog};

/// A watchdog held in module reset is never accessed.
pub fn service(reset: &ResetManager, watchdog0: &mut Watchdog0, watchdog1: &mut Watchdog1) {
    let resets = reset.peripheral.get();
    for (registers, mask) in [
        (&**watchdog0, reset_manager::PERMODRST_WATCHDOG0),
        (&**watchdog1, reset_manager::PERMODRST_WATCHDOG1),
    ] {
        if resets & mask == 0 && registers.control.is_set(watchdog::Control::ENABLE) {
            registers.restart.write(watchdog::Restart::KEY.val(0x76));
        }
    }
}
