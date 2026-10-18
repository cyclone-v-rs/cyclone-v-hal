//! DW APB watchdog: CR enable can latch until reset; CRR is a write-only command.
use tock_registers::{
    register_bitfields,
    register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};
register_bitfields![u32,
    pub Control [ ENABLE OFFSET(0) NUMBITS(1) [], RESPONSE OFFSET(1) NUMBITS(1) [] ],
    pub Restart [ KEY OFFSET(0) NUMBITS(8) [] ]
];
register_structs! {
    pub RegisterBlock {
        (0x000 => pub control: ReadWrite<u32, Control::Register>),
        (0x004 => pub timeout: ReadWrite<u32>),
        (0x008 => pub current: ReadOnly<u32>),
        (0x00c => pub restart: WriteOnly<u32, Restart::Register>),
        (0x010 => @END),
    }
}
