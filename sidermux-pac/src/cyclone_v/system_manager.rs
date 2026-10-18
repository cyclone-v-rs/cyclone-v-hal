//! HPS freeze controller, warm-reset I/O control, and all 207 pinmux slots.
use tock_registers::{
    register_bitfields,
    register_structs,
    registers::{ReadOnly, ReadWrite},
};
register_bitfields![u32,
    pub IoControl [ CONFIG OFFSET(0) NUMBITS(1) [], HOLD OFFSET(1) NUMBITS(1) [], TRISTATE OFFSET(2) NUMBITS(1) [], PULLUP OFFSET(3) NUMBITS(1) [], SLEW OFFSET(4) NUMBITS(1) [], DLL_RESET OFFSET(5) NUMBITS(1) [], OCT_RESET OFFSET(6) NUMBITS(1) [], REG_RESET OFFSET(7) NUMBITS(1) [], CAL_START OFFSET(8) NUMBITS(1) [] ],
    pub RomControl [ WARM_RESET_IO OFFSET(1) NUMBITS(1) [] ],
    pub Pinmux [ SELECT OFFSET(0) NUMBITS(2) [] ]
];
register_structs! {
    pub RegisterBlock {
        (0x000 => _reserved0),
        (0x040 => pub vio: [ReadWrite<u32, IoControl::Register>; 3]),
        (0x04c => _reserved1),
        (0x050 => pub hio: ReadWrite<u32, IoControl::Register>),
        (0x054 => pub freeze_source: ReadWrite<u32>),
        (0x058 => pub freeze_hardware: ReadOnly<u32>),
        (0x05c => _reserved2),
        (0x0c0 => pub rom_control: ReadWrite<u32, RomControl::Register>),
        (0x0c4 => _reserved3),
        (0x400 => pub pinmux: [ReadWrite<u32, Pinmux::Register>; 207]),
        (0x73c => @END),
    }
}

// Numeric encodings used for exact vendor transactions.
pub const PINMUX_DEDICATED_FUNCTION: u32 = 1;
pub const VIOCTRL: [usize; 3] = [0x40, 0x44, 0x48];
pub const WARMRSTCFGIO: u32 = 1 << 1;
pub const IO_SLEW: u32 = 0x10;
pub const IO_PULLUP_TRISTATE: u32 = 0x0c;
pub const IO_HOLD_CONFIG: u32 = 0x03;
pub const IO_ENABLE: u32 = 0x1f;
pub const HIO_DLL_RESET: u32 = 0x20;
pub const HIO_OCT_RESET: u32 = 0x40;
pub const HIO_REG_RESET: u32 = 0x80;
pub const HIO_CAL_START: u32 = 0x100;
pub const HIO_FROZEN: u32 = HIO_DLL_RESET | HIO_OCT_RESET | HIO_REG_RESET;
pub const HIO_THAWED: u32 = HIO_CAL_START | IO_ENABLE;
pub const HIO_CONTROL_MASK: u32 = 0x1ff;
