use core::mem::{align_of, offset_of, size_of};

use tock_registers::registers::InMemoryRegister;

use super::*;
use crate::prelude::*;

#[test]
fn handles_are_singletons_even_after_drop() {
    extern crate std;
    let successes = std::thread::scope(|scope| {
        let threads: std::vec::Vec<_> = (0..8)
            .map(|_| scope.spawn(|| Peripherals::take().is_some()))
            .collect();
        threads
            .into_iter()
            .map(|thread| thread.join().expect("claim thread"))
            .filter(|acquired| *acquired)
            .count()
    });
    assert_eq!(successes, 1);
    assert!(Peripherals::take().is_none());
}

macro_rules! layout {
    ($test:ident, $module:ident, $size:expr, $( $field:ident => $offset:expr ),+ $(,)?) => {
        #[test]
        fn $test() {
            assert_eq!(align_of::<$module::RegisterBlock>(), 4);
            assert_eq!(size_of::<$module::RegisterBlock>(), $size);
            $(assert_eq!(offset_of!($module::RegisterBlock, $field), $offset);)+
        }
    };
}
layout!(clock_layout, clock_manager, 0xec,
    ctrl => 0, bypass => 4, inter => 8, stat => 0x14,
    main_vco => 0x40, mpu => 0x48, main => 0x4c, debug_at => 0x50,
    main_qspi => 0x54, main_nandsdmmc => 0x58, cfg_user0 => 0x5c,
    main_enable => 0x60, main_divider => 0x64, debug_divider => 0x68,
    trace_divider => 0x6c, l4_source => 0x70, per_vco => 0x80,
    emac0 => 0x88, emac1 => 0x8c, per_qspi => 0x90, per_nandsdmmc => 0x94,
    per_base => 0x98, user1 => 0x9c, per_enable => 0xa0,
    per_divider => 0xa4, gpio_divider => 0xa8, per_source => 0xac,
    sdr_vco => 0xc0, ddr_dqs => 0xc8, ddr_2xdqs => 0xcc, ddr_dq => 0xd0,
    user2 => 0xd4, sdr_enable => 0xd8, altera_mpu => 0xe0,
    altera_main => 0xe4, altera_debug_at => 0xe8);
layout!(reset_layout, reset_manager, 0x24,
    stat => 0, peripheral => 0x14, peripheral2 => 0x18, misc => 0x20);
layout!(system_layout, system_manager, 0x73c,
    vio => 0x40, hio => 0x50, freeze_source => 0x54, freeze_hardware => 0x58,
    rom_control => 0xc0, pinmux => 0x400);
layout!(scan_layout, scan_manager, 0x20,
    stat => 0, enable => 4, single => 0x10, double => 0x14, triple => 0x18, quad => 0x1c);
layout!(timer_layout, timer, 0x14,
    load => 0, current => 4, control => 8, eoi => 0xc, interrupt => 0x10);
layout!(watchdog_layout, watchdog, 0x10,
    control => 0, timeout => 4, current => 8, restart => 0xc);
layout!(uart_layout, uart, 0x18,
    data => 0, interrupt => 4, fifo => 8, line_control => 0xc,
    modem_control => 0x10, line_status => 0x14);

#[test]
fn hardware_addresses_and_pinmux_extent() {
    assert_eq!(ClockManager::ptr() as usize, 0xffd04000);
    assert_eq!(ResetManager::ptr() as usize, 0xffd05000);
    assert_eq!(SystemManager::ptr() as usize, 0xffd08000);
    assert_eq!(ScanManager::ptr() as usize, 0xfff02000);
    assert_eq!(Osc1Timer0::ptr() as usize, 0xffd00000);
    assert_eq!(Watchdog0::ptr() as usize, 0xffd02000);
    assert_eq!(Watchdog1::ptr() as usize, 0xffd03000);
    assert_eq!(Uart0::ptr() as usize, 0xffc02000);
    assert_eq!(Uart1::ptr() as usize, 0xffc03000);
    assert_eq!(0x400 + 189 * 4, 0x6f4); // UART0USEFPGA
    assert_eq!(0x400 + 206 * 4, 0x738); // final pinmux register
}

#[test]
fn typed_fields_match_vendor_words() {
    let vco = InMemoryRegister::<u32, clock_manager::Vco::Register>::new(0);
    vco.write(clock_manager::Vco::NUMERATOR.val(63) + clock_manager::Vco::ENABLE::SET);
    assert_eq!(vco.get(), 0x1fa);
    assert_eq!(vco.read(clock_manager::Vco::NUMERATOR), 63);
    let reset = InMemoryRegister::<u32, reset_manager::PeripheralReset::Register>::new(0);
    reset.write(reset_manager::PeripheralReset::UART0::SET);
    assert_eq!(reset.get(), 1 << 16);
    let status = InMemoryRegister::<u32, scan_manager::Status::Register>::new(0xf0000000);
    assert!(status.is_set(scan_manager::Status::ACTIVE));
    assert_eq!(status.read(scan_manager::Status::FIFO_COUNT), 7);
    let timer = InMemoryRegister::<u32, timer::Control::Register>::new(0);
    timer.write(timer::Control::ENABLE::SET + timer::Control::MASK_INTERRUPT::SET);
    assert_eq!(timer.get(), 5);
    let rom = InMemoryRegister::<u32, system_manager::RomControl::Register>::new(0);
    rom.write(system_manager::RomControl::WARM_RESET_IO::SET);
    assert_eq!(rom.get(), 2);
}
