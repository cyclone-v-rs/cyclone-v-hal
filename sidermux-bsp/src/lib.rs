//! Board wiring, vendor handoff data, and initialization sequences.
#![no_std]

#[cfg(feature = "board-de1-soc")]
pub mod de1_soc;
#[cfg(feature = "board-de1-soc")]
mod initialization;
