//! UART0/UART1: DesignWare DW_apb_uart, 16550-compatible.
//!
//! Register map and bitfields per the Altera Cyclone V HPS Technical
//! Reference Manual and U-Boot `arch/arm/mach-socfpga` (gen5).
//!
//! Access contract: the block is word-addressed — each logical 16550
//! register occupies its own 32-bit slot (`REG_STRIDE` = 4 bytes), so
//! adjacent logical registers are NOT adjacent bytes. Data registers carry
//! their byte in bits 7:0. The divisor latches share slots with data and
//! interrupt registers; `LCR` bit 7 (DLAB) selects which meaning applies.

/// UART0 base address.
pub const UART0_BASE: usize = 0xFFC0_2000;
/// UART1 base address.
pub const UART1_BASE: usize = 0xFFC0_3000;
/// Distance between consecutive logical register slots.
pub const REG_STRIDE: usize = 4;

/// `DLAB = 0`: receive buffer (read) / transmit holding (write).
/// `DLAB = 1`: divisor latch low byte.
pub const REG_RBR_THR_DLL: usize = 0x0;
/// `DLAB = 0`: interrupt enable. `DLAB = 1`: divisor latch high byte.
pub const REG_IER_DLM: usize = 0x1 * REG_STRIDE;
/// `DLAB = 0`: interrupt identify (read) / FIFO control (write).
pub const REG_IIR_FCR: usize = 0x2 * REG_STRIDE;
/// Line control register (includes DLAB).
pub const REG_LCR: usize = 0x3 * REG_STRIDE;
/// Modem control register.
pub const REG_MCR: usize = 0x4 * REG_STRIDE;
/// Line status register.
pub const REG_LSR: usize = 0x5 * REG_STRIDE;

/// `LCR`: 8 data bits, no parity, 1 stop bit (8N1) with DLAB cleared.
pub const LCR_8N1: u32 = 0x3;
/// `LCR`: divisor latch access bit; selects divisor slots over data slots.
pub const LCR_DLAB: u32 = 0x80;

/// `FCR`: enable FIFOs.
pub const FCR_FIFO_ENABLE: u32 = 0x1;
/// `FCR`: clear the receive FIFO (self-clearing).
pub const FCR_RX_FIFO_RESET: u32 = 0x2;
/// `FCR`: clear the transmit FIFO (self-clearing).
pub const FCR_TX_FIFO_RESET: u32 = 0x4;

/// `IER`: all interrupt sources disabled.
pub const IER_DISABLE_ALL: u32 = 0x0;

/// `LSR`: data ready in receive buffer.
pub const LSR_DATA_READY: u32 = 0x1;
/// `LSR`: receive overrun error.
pub const LSR_OVERRUN: u32 = 0x2;
/// `LSR`: parity error.
pub const LSR_PARITY: u32 = 0x4;
/// `LSR`: framing error.
pub const LSR_FRAMING: u32 = 0x8;
/// `LSR`: break interrupt (received all-zero for a full frame time).
pub const LSR_BREAK: u32 = 0x10;
/// `LSR`: transmit holding register empty (accepts a new byte).
pub const LSR_THRE: u32 = 0x20;
/// `LSR`: transmitter empty (holding and shift register both idle).
pub const LSR_TRANSMITTER_EMPTY: u32 = 0x40;
