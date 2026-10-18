//! Cyclone V HPS register maps and single-owner peripheral handles.
//!
//! Layouts follow the Cyclone V HPS TRM (683126), register-map chapters,
//! and vendor U-Boot b56517573b02f683018e4599e77338e757c6a288. This is a
//! deliberately partial PAC covering the current boot/console peripherals.
//!
//! Acquire peripherals on core0 after runtime initialization. Core1 must
//! remain parked; this API does not establish multicore cache coherence.
//! MMIO must be identity mapped with device/strongly-ordered attributes.

//! Read-only registers cannot be written:
//! ```compile_fail
//! use sidermux_pac::{cyclone_v::Peripherals, prelude::*};
//! let p = Peripherals::take().unwrap();
//! p.uart0.line_status.set(0);
//! ```
//! Write-only commands cannot be read:
//! ```compile_fail
//! use sidermux_pac::{cyclone_v::Peripherals, prelude::*};
//! let p = Peripherals::take().unwrap();
//! let value = p.watchdog0.restart.get();
//! ```
//! Acknowledgement registers do not allow ordinary read-modify-write:
//! ```compile_fail
//! use sidermux_pac::{cyclone_v::{Peripherals, clock_manager}, prelude::*};
//! let p = Peripherals::take().unwrap();
//! p.clock_manager.inter.modify(clock_manager::InterruptAck::MAIN_LOST::SET);
//! ```
//! UART FIFO aliases cannot be modified by reading the interrupt identity:
//! ```compile_fail
//! use sidermux_pac::{cyclone_v::{Peripherals, uart}, prelude::*};
//! let p = Peripherals::take().unwrap();
//! p.uart0.fifo.modify(uart::FifoControl::ENABLE::SET);
//! ```
//! Field identities cannot be mixed across registers:
//! ```compile_fail
//! use sidermux_pac::{cyclone_v::{Peripherals, watchdog}, prelude::*};
//! let p = Peripherals::take().unwrap();
//! p.uart0.line_control.write(watchdog::Control::ENABLE::SET);
//! ```
//! Handles cannot cross core/thread boundaries:
//! ```compile_fail
//! use sidermux_pac::cyclone_v::Peripherals;
//! let p = Peripherals::take().unwrap();
//! std::thread::spawn(move || drop(p.uart0));
//! ```
//! Borrowing a register block keeps its handle borrowed:
//! ```compile_fail
//! use sidermux_pac::{cyclone_v::Peripherals, prelude::*};
//! let p = Peripherals::take().unwrap();
//! let registers = &*p.uart0;
//! let owned = p.uart0;
//! let value = registers.line_status.get();
//! ```

use core::{
    marker::PhantomData,
    ops::Deref,
    sync::atomic::{AtomicBool, Ordering},
};

pub mod clock_manager;
pub mod reset_manager;
pub mod scan_manager;
pub mod system_manager;
pub mod timer;
pub mod uart;
pub mod watchdog;

static TAKEN: AtomicBool = AtomicBool::new(false);

// OCRAM boot cannot rely on a successful external exclusive transaction.
// Serialize plain atomic loads/stores on core0 by masking IRQ and FIQ. Core1
// cannot acquire handles; this is not a multicore synchronization primitive.
#[cfg(target_arch = "arm")]
fn claim() -> bool {
    let core_id: u32;
    // SAFETY: this SoC uses privileged Cortex-A9 startup. Reading MPIDR
    // identifies the caller and performs no peripheral MMIO.
    unsafe {
        core::arch::asm!("mrc p15, 0, {id}, c0, c0, 5", id = out(reg) core_id, options(nostack, preserves_flags));
    }
    if core_id & 0xff != 0 {
        return false;
    }
    let saved: u32;
    // No nomem option: these blocks also prevent compiler memory motion
    // across the critical section. Restore the caller's exact mask state.
    unsafe {
        core::arch::asm!("mrs {saved}, cpsr", "cpsid if", saved = out(reg) saved, options(nostack, preserves_flags));
    }
    let acquired = !TAKEN.load(Ordering::Acquire);
    if acquired {
        TAKEN.store(true, Ordering::Release);
    }
    unsafe {
        core::arch::asm!("msr cpsr_c, {saved}", saved = in(reg) saved, options(nostack, preserves_flags));
    }
    acquired
}

#[cfg(not(target_arch = "arm"))]
fn claim() -> bool {
    TAKEN
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

macro_rules! peripheral {
    ($name:ident, $module:ident, $address:expr) => {
        #[doc = concat!(stringify!($name), " peripheral token; neither Send nor Sync.")]
        #[derive(Debug)]
        pub struct $name {
            _exclusive: PhantomData<*mut ()>,
        }
        impl $name {
            fn new() -> Self {
                Self {
                    _exclusive: PhantomData,
                }
            }
            /// Raw register pointer. Dereferencing bypasses ownership checks.
            pub const fn ptr() -> *const $module::RegisterBlock {
                $address as *const _
            }
            /// Bypass peripheral ownership.
            ///
            /// # Safety
            /// No other handle or register reference to this peripheral may
            /// be in use. The caller must preserve exclusive access.
            pub unsafe fn steal() -> Self {
                TAKEN.store(true, Ordering::Release);
                Self::new()
            }
        }
        impl Deref for $name {
            type Target = $module::RegisterBlock;
            fn deref(&self) -> &Self::Target {
                // SAFETY: safe construction is singleton-only, and the
                // pointer addresses this SoC's static MMIO register block.
                unsafe { &*Self::ptr() }
            }
        }
    };
}
peripheral!(ClockManager, clock_manager, 0xffd04000);
peripheral!(ResetManager, reset_manager, 0xffd05000);
peripheral!(SystemManager, system_manager, 0xffd08000);
peripheral!(ScanManager, scan_manager, 0xfff02000);
peripheral!(Osc1Timer0, timer, 0xffd00000);
peripheral!(Watchdog0, watchdog, 0xffd02000);
peripheral!(Watchdog1, watchdog, 0xffd03000);
peripheral!(Uart0, uart, 0xffc02000);
peripheral!(Uart1, uart, 0xffc03000);

/// All supported peripheral handles. Dropping handles never releases the
/// singleton guard. Taking tokens performs no hardware accesses.
///
/// Handles cannot be duplicated:
/// ```compile_fail
/// use sidermux_pac::cyclone_v::Peripherals;
/// let p = Peripherals::take().unwrap();
/// let uart = p.uart0;
/// let duplicate = p.uart0;
/// # let _ = (uart, duplicate);
/// ```
pub struct Peripherals {
    pub clock_manager: ClockManager,
    pub reset_manager: ResetManager,
    pub system_manager: SystemManager,
    pub scan_manager: ScanManager,
    pub timer0: Osc1Timer0,
    pub watchdog0: Watchdog0,
    pub watchdog1: Watchdog1,
    pub uart0: Uart0,
    pub uart1: Uart1,
}
impl Peripherals {
    /// Acquire ownership once, from the initialized privileged core0 runtime.
    /// IRQ/FIQ are briefly masked and restored; no OCRAM exclusive RMW is used.
    pub fn take() -> Option<Self> {
        claim().then(Self::new)
    }
    fn new() -> Self {
        Self {
            clock_manager: ClockManager::new(),
            reset_manager: ResetManager::new(),
            system_manager: SystemManager::new(),
            scan_manager: ScanManager::new(),
            timer0: Osc1Timer0::new(),
            watchdog0: Watchdog0::new(),
            watchdog1: Watchdog1::new(),
            uart0: Uart0::new(),
            uart1: Uart1::new(),
        }
    }
    /// Bypass the singleton guard.
    ///
    /// # Safety
    /// No other peripheral handles or borrowed register references may be
    /// in use. Caller must enforce exclusivity and core0 startup conditions.
    pub unsafe fn steal() -> Self {
        TAKEN.store(true, Ordering::Release);
        Self::new()
    }
}

impl uart::sealed::Sealed for Uart0 {}
impl uart::sealed::Sealed for Uart1 {}
impl uart::Instance for Uart0 {
    fn registers(&self) -> &uart::RegisterBlock {
        self
    }
}
impl uart::Instance for Uart1 {
    fn registers(&self) -> &uart::RegisterBlock {
        self
    }
}

#[cfg(test)]
mod tests;
