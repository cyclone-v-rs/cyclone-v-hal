//! Minimal register definitions for the Cyclone V HPS.
//!
//! Each module states where registers are and what their fields mean;
//! operations on them live in the surrounding HAL modules.

pub mod clock_manager;
pub mod reset_manager;
pub mod system_manager;
pub mod uart;
