//! OSC1 timer0 remains independent of PLL reconfiguration.
use sidermux_pac::prelude::*;

use super::pac::{Osc1Timer0, ResetManager, reset_manager, timer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerError {
    DelayTooLong,
    Timeout,
}

pub fn init(timer: &mut Osc1Timer0, reset: &mut ResetManager) {
    reset
        .peripheral
        .modify(reset_manager::PeripheralReset::TIMER0::CLEAR);
    timer.control.set(0);
    timer.load.set(u32::MAX);
    timer
        .control
        .write(timer::Control::ENABLE::SET + timer::Control::MASK_INTERRUPT::SET);
}
/// Bounded delay; the callback keeps inherited watchdogs serviced.
pub fn delay_us(
    timer: &Osc1Timer0,
    us: u32,
    osc_hz: u32,
    mut service: impl FnMut(),
) -> Result<(), TimerError> {
    let ticks = (u64::from(us) * u64::from(osc_hz)).div_ceil(1_000_000);
    if ticks > u64::from(u32::MAX) {
        return Err(TimerError::DelayTooLong);
    }
    let start = timer.current.get();
    for i in 0..2_000_000 {
        if start.wrapping_sub(timer.current.get()) >= ticks as u32 {
            return Ok(());
        }
        if i % 256 == 0 {
            service();
        }
    }
    Err(TimerError::Timeout)
}
