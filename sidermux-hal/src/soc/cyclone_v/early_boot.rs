//! Owned infrastructure kept alive across boot initialization and idle.
//! No singleton acquisition or hidden peripheral stealing occurs here.
use super::{
    pac::{ClockManager, Osc1Timer0, ResetManager, Watchdog0, Watchdog1},
    reset,
    timer,
    watchdog,
};

pub struct BootSupport {
    reset: ResetManager,
    timer: Osc1Timer0,
    watchdog0: Watchdog0,
    watchdog1: Watchdog1,
}
impl BootSupport {
    /// Assemble owned handles without touching hardware.
    pub fn new(
        reset: ResetManager,
        timer: Osc1Timer0,
        watchdog0: Watchdog0,
        watchdog1: Watchdog1,
    ) -> Self {
        Self {
            reset,
            timer,
            watchdog0,
            watchdog1,
        }
    }
    pub fn service_watchdogs(&mut self) {
        watchdog::service(&self.reset, &mut self.watchdog0, &mut self.watchdog1);
    }
    pub fn reset_cause(&self) -> u32 {
        reset::cause(&self.reset)
    }
    pub fn init_timer(&mut self) {
        timer::init(&mut self.timer, &mut self.reset);
    }
    pub fn prepare_io(&mut self) {
        reset::prepare_io(&mut self.reset);
    }
    pub fn release_uart0(&mut self, clocks: &mut ClockManager) {
        reset::release_uart0(&mut self.reset, clocks);
    }
    pub fn uart0_released(&self) -> bool {
        reset::uart0_released(&self.reset)
    }
    pub fn delay_us(&mut self, us: u32, osc_hz: u32) -> Result<(), timer::TimerError> {
        timer::delay_us(&self.timer, us, osc_hz, || {
            watchdog::service(&self.reset, &mut self.watchdog0, &mut self.watchdog1)
        })
    }
}
