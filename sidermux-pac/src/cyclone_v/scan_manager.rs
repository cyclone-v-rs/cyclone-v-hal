//! Scan-manager status, port enable and FIFO apertures. Only the IOCSR
//! subset is exposed; FPGA TAP control commands are not implemented.
use tock_registers::{
    register_bitfields,
    register_structs,
    registers::{Aliased, ReadOnly, ReadWrite},
};
register_bitfields![u32,
    pub Status [ FIFO_COUNT OFFSET(28) NUMBITS(3) [], ACTIVE OFFSET(31) NUMBITS(1) [] ],
    pub Enable [ PORTS OFFSET(0) NUMBITS(8) [] ],
    pub Fifo8Read [ DATA OFFSET(0) NUMBITS(8) [] ],
    pub Fifo8Write [ DATA OFFSET(0) NUMBITS(8) [] ],
    pub Fifo16Read [ DATA OFFSET(0) NUMBITS(16) [] ],
    pub Fifo16Write [ DATA OFFSET(0) NUMBITS(16) [] ],
    pub Fifo24Read [ DATA OFFSET(0) NUMBITS(24) [] ],
    pub Fifo24Write [ DATA OFFSET(0) NUMBITS(24) [] ],
    pub Fifo32Read [ DATA OFFSET(0) NUMBITS(32) [] ],
    pub Fifo32Write [ DATA OFFSET(0) NUMBITS(32) [] ]
];
register_structs! {
    pub RegisterBlock {
        (0x000 => pub stat: ReadOnly<u32, Status::Register>),
        (0x004 => pub enable: ReadWrite<u32, Enable::Register>),
        (0x008 => _reserved0),
        (0x010 => pub single: Aliased<u32, Fifo8Read::Register, Fifo8Write::Register>),
        (0x014 => pub double: Aliased<u32, Fifo16Read::Register, Fifo16Write::Register>),
        (0x018 => pub triple: Aliased<u32, Fifo24Read::Register, Fifo24Write::Register>),
        (0x01c => pub quad: Aliased<u32, Fifo32Read::Register, Fifo32Write::Register>),
        (0x020 => @END),
    }
}
