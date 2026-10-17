//! Board configuration on top of a SoC: installed DDR geometry and hardware
//! settings. Board features are expected to depend on their SoC feature;
//! board initialization stays distinct from SoC and architecture code.
//!
//! Exactly one board feature must be selected by target-dependent builds;
//! the HAL itself builds without any board.

#[cfg(feature = "board-de1-soc")]
pub mod de1_soc;
