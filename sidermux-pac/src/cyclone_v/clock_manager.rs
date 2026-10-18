//! Cyclone V clock manager. Interrupt acknowledgements and phase/exit
//! commands intentionally have distinct read/write types: no ordinary RMW.
use tock_registers::{
    register_bitfields,
    register_structs,
    registers::{Aliased, ReadOnly, ReadWrite},
};
register_bitfields![u32,
    pub ControlRead [ SAFE OFFSET(0) NUMBITS(1) [], ],
    pub ControlWrite [ EXIT_SAFE OFFSET(0) NUMBITS(1) [], ],
    pub Bypass [ MAIN OFFSET(0) NUMBITS(1) [], SDR OFFSET(1) NUMBITS(1) [], PER OFFSET(3) NUMBITS(1) [] ],
    pub InterruptStatus [ MAIN_LOCKED OFFSET(6) NUMBITS(1) [], PER_LOCKED OFFSET(7) NUMBITS(1) [], SDR_LOCKED OFFSET(8) NUMBITS(1) [] ],
    pub InterruptAck [ MAIN_LOST OFFSET(3) NUMBITS(1) [], PER_LOST OFFSET(4) NUMBITS(1) [], SDR_LOST OFFSET(5) NUMBITS(1) [] ],
    pub Status [ BUSY OFFSET(0) NUMBITS(1) [] ],
    pub Vco [ ENABLE OFFSET(1) NUMBITS(1) [], NUMERATOR OFFSET(3) NUMBITS(13) [], DENOMINATOR OFFSET(16) NUMBITS(6) [], SOURCE OFFSET(22) NUMBITS(2) [], OUTPUT_RESET OFFSET(24) NUMBITS(1) [] ],
    pub Counter [ COUNT OFFSET(0) NUMBITS(9) [] ],
    pub SdramRead [ COUNT OFFSET(0) NUMBITS(9) [], PHASE OFFSET(9) NUMBITS(3) [] ],
    pub SdramWrite [ COUNT OFFSET(0) NUMBITS(9) [], PHASE OFFSET(9) NUMBITS(3) [] ],
    pub MainEnable [ L4SP OFFSET(1) NUMBITS(1) [] ],
    pub MainDivider [ L4SP OFFSET(7) NUMBITS(3) [] ],
    pub L4Source [ L4SP OFFSET(1) NUMBITS(1) [Main = 0, Peripheral = 1] ]
];
register_structs! {
    pub RegisterBlock {
        (0x000 => pub ctrl: Aliased<u32, ControlRead::Register, ControlWrite::Register>),
        (0x004 => pub bypass: ReadWrite<u32, Bypass::Register>),
        (0x008 => pub inter: Aliased<u32, InterruptStatus::Register, InterruptAck::Register>),
        (0x00c => _reserved0),
        (0x014 => pub stat: ReadOnly<u32, Status::Register>),
        (0x018 => _reserved1),
        (0x040 => pub main_vco: ReadWrite<u32, Vco::Register>),
        (0x044 => _reserved2),
        (0x048 => pub mpu: ReadWrite<u32, Counter::Register>),
        (0x04c => pub main: ReadWrite<u32, Counter::Register>),
        (0x050 => pub debug_at: ReadWrite<u32, Counter::Register>),
        (0x054 => pub main_qspi: ReadWrite<u32, Counter::Register>),
        (0x058 => pub main_nandsdmmc: ReadWrite<u32, Counter::Register>),
        (0x05c => pub cfg_user0: ReadWrite<u32, Counter::Register>),
        (0x060 => pub main_enable: ReadWrite<u32, MainEnable::Register>),
        (0x064 => pub main_divider: ReadWrite<u32, MainDivider::Register>),
        (0x068 => pub debug_divider: ReadWrite<u32>),
        (0x06c => pub trace_divider: ReadWrite<u32>),
        (0x070 => pub l4_source: ReadWrite<u32, L4Source::Register>),
        (0x074 => _reserved3),
        (0x080 => pub per_vco: ReadWrite<u32, Vco::Register>),
        (0x084 => _reserved4),
        (0x088 => pub emac0: ReadWrite<u32, Counter::Register>),
        (0x08c => pub emac1: ReadWrite<u32, Counter::Register>),
        (0x090 => pub per_qspi: ReadWrite<u32, Counter::Register>),
        (0x094 => pub per_nandsdmmc: ReadWrite<u32, Counter::Register>),
        (0x098 => pub per_base: ReadWrite<u32, Counter::Register>),
        (0x09c => pub user1: ReadWrite<u32, Counter::Register>),
        (0x0a0 => pub per_enable: ReadWrite<u32>),
        (0x0a4 => pub per_divider: ReadWrite<u32>),
        (0x0a8 => pub gpio_divider: ReadWrite<u32>),
        (0x0ac => pub per_source: ReadWrite<u32>),
        (0x0b0 => _reserved5),
        (0x0c0 => pub sdr_vco: ReadWrite<u32, Vco::Register>),
        (0x0c4 => _reserved6),
        (0x0c8 => pub ddr_dqs: Aliased<u32, SdramRead::Register, SdramWrite::Register>),
        (0x0cc => pub ddr_2xdqs: Aliased<u32, SdramRead::Register, SdramWrite::Register>),
        (0x0d0 => pub ddr_dq: Aliased<u32, SdramRead::Register, SdramWrite::Register>),
        (0x0d4 => pub user2: Aliased<u32, SdramRead::Register, SdramWrite::Register>),
        (0x0d8 => pub sdr_enable: ReadWrite<u32>),
        (0x0dc => _reserved7),
        (0x0e0 => pub altera_mpu: ReadWrite<u32, Counter::Register>),
        (0x0e4 => pub altera_main: ReadWrite<u32, Counter::Register>),
        (0x0e8 => pub altera_debug_at: ReadWrite<u32, Counter::Register>),
        (0x0ec => @END),
    }
}

// Numeric encodings used for exact vendor transactions.
pub const CTRL_SAFEMODE: u32 = 1 << 0;
pub const BYPASS_MAINPLL: u32 = 1 << 0;
pub const STAT_BUSY: u32 = 1 << 0;
pub const INTER_MAINPLLLOCKED: u32 = 1 << 6;
pub const MAINPLL_VCO_EN: u32 = 1 << 1;
pub const MAINPLL_EN_L4SPCLK: u32 = 1 << 1;
pub const L4SRC_L4SP_OFFSET: u32 = 1;
pub const L4SRC_L4SP_MAIN: u32 = 0x0;
pub const L4SRC_L4SP_PERPLL: u32 = 0x1;
pub const MAINDIV_L4SPCLK_OFFSET: u32 = 7;
pub const MAINDIV_L4SPCLK_MASK: u32 = 0x7;
pub const VCO_DENOM_OFFSET: u32 = 16;
pub const VCO_DENOM_MASK: u32 = 0x3F;
pub const VCO_NUMER_OFFSET: u32 = 3;
pub const VCO_NUMER_MASK: u32 = 0x1FFF;
pub const PERPLL_VCO_SSRC_OFFSET: u32 = 22;
pub const PERPLL_VCO_SSRC_MASK: u32 = 0x3;
pub const SSRC_EOSC1: u32 = 0x0;
pub const SSRC_EOSC2: u32 = 0x1;
pub const SSRC_F2S: u32 = 0x2;
pub const MAINPLL_EN_EARLY: u32 = 0x2f4;
pub const PERPLL_EN_NAND: u32 = 1 << 10;
pub const BYPASS_ALL: u32 = 0xb;
pub const VCO_RESET: u32 = 0x0001_000d;
pub const PERPLL_SRC_RESET: u32 = 0x15;
pub const INTER_ALL_LOCKED: u32 = 0x1c0;
pub const INTER_LOST_LOCK: u32 = 0x38;
pub const VCO_OUTRESETALL: u32 = 1 << 24;
pub const SDRPLL_COUNTER_MASK: u32 = 0x1ff;
pub const SDRPLL_PHASE_MASK: u32 = 0xe00;
