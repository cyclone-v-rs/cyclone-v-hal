//! Typed MMIO register definitions. Board configuration and initialization
//! policies belong in the HAL. All SoC modules share the register access
//! traits in [`prelude`], while retaining their own hardware layouts.
#![no_std]

pub use tock_registers::registers;

pub mod prelude {
    pub use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};
}

#[cfg(feature = "cyclone-v")]
pub mod cyclone_v;
