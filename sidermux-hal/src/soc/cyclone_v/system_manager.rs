//! Software freeze FSM, warm-reset IO protection, and generated pinmux.
use sidermux_pac::prelude::*;

use super::{
    early_boot::BootSupport,
    pac::{SystemManager, system_manager as pac},
};
fn update(
    register: &sidermux_pac::registers::ReadWrite<u32, pac::IoControl::Register>,
    clear: u32,
    set: u32,
) {
    register.set((register.get() & !clear) | set);
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoError {
    Timer(super::timer::TimerError),
    Freeze,
    Thaw,
    Pinmux { index: usize, actual: u32 },
}
#[derive(Debug, Clone, Copy)]
struct FreezeState {
    pub vio: [u32; 3],
    pub hio: u32,
    pub source: u32,
}
fn freeze_state(manager: &SystemManager) -> FreezeState {
    FreezeState {
        vio: core::array::from_fn(|i| manager.vio[i].get()),
        hio: manager.hio.get(),
        source: manager.freeze_source.get(),
    }
}
fn delay(support: &mut BootSupport, us: u32, osc: u32) -> Result<(), IoError> {
    support.delay_us(us, osc).map_err(IoError::Timer)
}
pub fn freeze(
    manager: &mut SystemManager,
    support: &mut BootSupport,
    osc: u32,
) -> Result<(), IoError> {
    manager.freeze_source.set(0);
    for (i, register) in manager
        .vio
        .iter()
        .chain(core::iter::once(&manager.hio))
        .enumerate()
    {
        update(register, pac::IO_SLEW | pac::IO_PULLUP_TRISTATE, 0);
        delay(support, 1, osc)?; // exceeds the 20 ns minimum
        update(
            register,
            pac::IO_HOLD_CONFIG,
            if i == 3 {
                pac::HIO_REG_RESET | pac::HIO_OCT_RESET
            } else {
                0
            },
        );
    }
    update(&manager.hio, pac::HIO_CAL_START, pac::HIO_DLL_RESET);
    let s = freeze_state(manager);
    if s.vio.iter().any(|v| v & pac::IO_ENABLE != 0)
        || s.hio & pac::HIO_CONTROL_MASK != pac::HIO_FROZEN
        || s.source != 0
    {
        return Err(IoError::Freeze);
    }
    Ok(())
}
pub fn prepare_hio_scan(manager: &mut SystemManager) {
    update(&manager.hio, pac::HIO_DLL_RESET, 0);
}
pub fn thaw(
    manager: &mut SystemManager,
    support: &mut BootSupport,
    osc: u32,
) -> Result<(), IoError> {
    manager.freeze_source.set(0);
    for register in &manager.vio {
        update(register, 0, pac::IO_HOLD_CONFIG);
        delay(support, 1, osc)?;
        update(register, 0, pac::IO_PULLUP_TRISTATE);
        delay(support, 1, osc)?;
        update(register, 0, pac::IO_SLEW);
    }
    update(&manager.hio, pac::HIO_DLL_RESET, 0);
    delay(support, 1, osc)?;
    update(&manager.hio, 0, pac::HIO_CAL_START);
    delay(
        support,
        (1_000_000_000u64.div_ceil(u64::from(osc))) as u32,
        osc,
    )?;
    update(&manager.hio, pac::HIO_OCT_RESET, pac::IO_HOLD_CONFIG);
    delay(support, 2, osc)?;
    update(&manager.hio, 0, pac::IO_PULLUP_TRISTATE);
    delay(support, 1, osc)?;
    update(&manager.hio, pac::HIO_REG_RESET, 0);
    delay(support, 1, osc)?;
    update(&manager.hio, 0, pac::IO_SLEW);
    verify_thawed(manager)
}
fn verify_thawed(manager: &SystemManager) -> Result<(), IoError> {
    let s = freeze_state(manager);
    if s.source != 0
        || s.vio.iter().any(|v| v & pac::IO_ENABLE != pac::IO_ENABLE)
        || s.hio & pac::HIO_CONTROL_MASK != pac::HIO_THAWED
    {
        return Err(IoError::Thaw);
    }
    Ok(())
}
/// Allow Boot ROM to restore IO after a warm reset during either programming
/// window. On error leave the bit set: the configuration may be incomplete.
pub fn warm_reset_io(manager: &mut SystemManager, enable: bool) {
    manager
        .rom_control
        .modify(pac::RomControl::WARM_RESET_IO.val(u32::from(enable)));
}
pub fn configure_pinmux(manager: &mut SystemManager, table: &[u8]) -> Result<(), IoError> {
    for (i, &value) in table.iter().enumerate() {
        manager.pinmux[i].set(u32::from(value));
    }
    verify_pinmux(manager, table)
}
fn verify_pinmux(manager: &SystemManager, table: &[u8]) -> Result<(), IoError> {
    for (i, &value) in table.iter().enumerate() {
        let actual = manager.pinmux[i].get();
        if actual != u32::from(value) {
            return Err(IoError::Pinmux { index: i, actual });
        }
    }
    Ok(())
}
