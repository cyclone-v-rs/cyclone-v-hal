//! DW APB UART0/1. Aliased slots expose distinct read/write identities and
//! never permit ordinary RMW. DLAB selects data/IER versus divisor latches;
//! that mode must be established by the HAL before accessing either alias.
use tock_registers::{
    register_bitfields,
    register_structs,
    registers::{Aliased, ReadOnly, ReadWrite},
};
register_bitfields![u32,
    pub DataRead [ BYTE OFFSET(0) NUMBITS(8) [] ],
    pub DataWrite [ BYTE OFFSET(0) NUMBITS(8) [] ],
    pub InterruptRead [ BYTE OFFSET(0) NUMBITS(8) [] ],
    pub InterruptWrite [ BYTE OFFSET(0) NUMBITS(8) [] ],
    pub Identify [ VALUE OFFSET(0) NUMBITS(8) [] ],
    pub FifoControl [ ENABLE OFFSET(0) NUMBITS(1) [], RX_RESET OFFSET(1) NUMBITS(1) [], TX_RESET OFFSET(2) NUMBITS(1) [] ],
    pub LineControl [ WORD_LENGTH OFFSET(0) NUMBITS(2) [], DLAB OFFSET(7) NUMBITS(1) [] ],
    pub LineStatus [ DATA_READY OFFSET(0) NUMBITS(1) [], OVERRUN OFFSET(1) NUMBITS(1) [], PARITY OFFSET(2) NUMBITS(1) [], FRAMING OFFSET(3) NUMBITS(1) [], BREAK OFFSET(4) NUMBITS(1) [], THRE OFFSET(5) NUMBITS(1) [], EMPTY OFFSET(6) NUMBITS(1) [] ]
];
register_structs! {
    pub RegisterBlock {
        (0x000 => pub data: Aliased<u32, DataRead::Register, DataWrite::Register>),
        (0x004 => pub interrupt: Aliased<u32, InterruptRead::Register, InterruptWrite::Register>),
        (0x008 => pub fifo: Aliased<u32, Identify::Register, FifoControl::Register>),
        (0x00c => pub line_control: ReadWrite<u32, LineControl::Register>),
        (0x010 => pub modem_control: ReadWrite<u32>),
        (0x014 => pub line_status: ReadOnly<u32, LineStatus::Register>),
        (0x018 => @END),
    }
}

// Numeric encodings used for exact vendor transactions.
pub const LCR_8N1: u32 = 0x3;
pub const LCR_DLAB: u32 = 0x80;
pub const FCR_FIFO_ENABLE: u32 = 0x1;
pub const FCR_RX_FIFO_RESET: u32 = 0x2;
pub const FCR_TX_FIFO_RESET: u32 = 0x4;
pub const IER_DISABLE_ALL: u32 = 0x0;
pub const LSR_DATA_READY: u32 = 0x1;
pub const LSR_OVERRUN: u32 = 0x2;
pub const LSR_PARITY: u32 = 0x4;
pub const LSR_FRAMING: u32 = 0x8;
pub const LSR_BREAK: u32 = 0x10;
pub const LSR_THRE: u32 = 0x20;
pub const LSR_TRANSMITTER_EMPTY: u32 = 0x40;

pub(crate) mod sealed {
    pub trait Sealed {}
}
/// UART instances supported by the HAL. Implemented only by this PAC.
pub trait Instance: sealed::Sealed {
    fn registers(&self) -> &RegisterBlock;
}
