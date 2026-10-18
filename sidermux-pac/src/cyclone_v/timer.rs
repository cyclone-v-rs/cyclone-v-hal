//! OSC1 DW APB timer0. Reading EOI acknowledges an interrupt; no implicit reads.
use tock_registers::{
    register_bitfields,
    register_structs,
    registers::{ReadOnly, ReadWrite},
};
register_bitfields![u32,
    pub Control [ ENABLE OFFSET(0) NUMBITS(1) [], MODE OFFSET(1) NUMBITS(1) [], MASK_INTERRUPT OFFSET(2) NUMBITS(1) [] ]
];
register_structs! {
    pub RegisterBlock {
        (0x000 => pub load: ReadWrite<u32>),
        (0x004 => pub current: ReadOnly<u32>),
        (0x008 => pub control: ReadWrite<u32, Control::Register>),
        (0x00c => pub eoi: ReadOnly<u32>),
        (0x010 => pub interrupt: ReadOnly<u32>),
        (0x014 => @END),
    }
}
