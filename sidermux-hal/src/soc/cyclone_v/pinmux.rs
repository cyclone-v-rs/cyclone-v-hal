//! Pinmux mechanism: writing function-select entries in the system manager
//! pinmux table. Which entries a board selects is BSP policy.

use core::ptr;

use super::pac::system_manager;

/// Writes one pinmux table entry, selecting the function of a pin.
///
/// This is the HAL mechanism; callers (boards) own the choice of index and
/// value. Values outside the documented select range are rejected to catch
/// swapped arguments.
pub fn set_entry(index: usize, function: u32) -> Result<(), PinmuxError> {
    if function > system_manager::PINMUX_DEDICATED_FUNCTION {
        return Err(PinmuxError::InvalidFunction(function));
    }
    unsafe {
        ptr::write_volatile(
            (system_manager::PINMUX_BASE + index * 4) as *mut u32,
            function,
        );
    }
    Ok(())
}

/// Pinmux mechanism errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinmuxError {
    /// Function select outside the documented range (0 = GPIO, 1 = dedicated).
    InvalidFunction(u32),
}
