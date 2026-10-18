//! Reset manager. Status reads and W1C acknowledgements are distinct.
use tock_registers::{
    register_bitfields,
    register_structs,
    registers::{Aliased, ReadWrite},
};
register_bitfields![u32,
    pub ResetStatus [ SOURCES OFFSET(0) NUMBITS(32) [] ],
    pub ResetAck [ CLEAR OFFSET(0) NUMBITS(32) [] ],
    pub PeripheralReset [ WATCHDOG0 OFFSET(6) NUMBITS(1) [], WATCHDOG1 OFFSET(7) NUMBITS(1) [], TIMER0 OFFSET(8) NUMBITS(1) [], UART0 OFFSET(16) NUMBITS(1) [] ],
    pub MiscReset [ SCANMGR OFFSET(11) NUMBITS(1) [] ]
];
register_structs! {
    pub RegisterBlock {
        (0x000 => pub stat: Aliased<u32, ResetStatus::Register, ResetAck::Register>),
        (0x004 => _reserved0),
        (0x014 => pub peripheral: ReadWrite<u32, PeripheralReset::Register>),
        (0x018 => pub peripheral2: ReadWrite<u32>),
        (0x01c => _reserved1),
        (0x020 => pub misc: ReadWrite<u32, MiscReset::Register>),
        (0x024 => @END),
    }
}

// Numeric encodings used for exact vendor transactions.
pub const PERMODRST_UART0: u32 = 1 << 16;
pub const PERMODRST_WATCHDOG0: u32 = 1 << 6;
pub const PERMODRST_WATCHDOG1: u32 = 1 << 7;
pub const PERMODRST_OSC1TIMER0: u32 = 1 << 8;
pub const MISCMODRST_SCANMGR: u32 = 1 << 11;
