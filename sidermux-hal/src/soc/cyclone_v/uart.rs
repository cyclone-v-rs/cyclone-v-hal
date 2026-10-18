//! Polling UART driver for the DesignWare/16550-compatible HPS UARTs.
//!
//! The caller supplies an owned UART peripheral handle and register-derived L4 SP frequency,
//! after configuring board pins, releasing reset, and thawing I/O. Interrupts
//! and hardware flow control stay disabled. All transmit waits are bounded.

use sidermux_pac::prelude::*;

use super::pac::uart::{self as pac, Instance};

/// Bounded-polling budget in CPU iterations. Not time-based: until a timer
/// exists this budget only has to exceed the slowest expected hardware event
/// (one byte at the lowest plausible baud) by orders of magnitude.
const POLL_ITERATION_BUDGET: u32 = 1_000_000;

/// Maximum accepted error between the requested and achievable baud, in
/// parts per million (2%).
const MAX_BAUD_ERROR_PPM: u64 = 20_000;

/// A configured polling UART.
#[derive(Debug)]
pub struct Uart<P: Instance> {
    peripheral: P,
}

/// Driver errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UartError {
    /// Bounded polling budget exhausted; the expected hardware event did not
    /// happen. Points at clock, reset, or pin configuration.
    Timeout,
    /// The requested baud needs a divisor outside the supported range or
    /// misses the error budget at the supplied reference frequency.
    BaudUnsupported { requested: u32 },
    /// Received a line break (all-zero frame).
    Break,
    /// Received byte with a framing error.
    Framing,
    /// Received byte with a parity error.
    Parity,
    /// Receive data lost to overrun.
    Overrun,
}

impl<P: Instance> Uart<P> {
    /// Configures the UART for 8N1 polling with FIFOs enabled.
    ///
    /// The peripheral token selects UART0 or UART1. Pins, clocks and reset must
    /// already be configured by the caller.
    pub fn new(peripheral: P, uart_clk_hz: u32, baud: u32) -> Result<Self, UartError> {
        let (divisor, _) = divisor_for(uart_clk_hz, baud)?;
        Ok(Self::configure_regs(peripheral, divisor))
    }

    /// Register sequence: interrupts off, DLAB
    /// write of the divisor, 8N1, FIFO enable + clear, flow control off.
    fn configure_regs(peripheral: P, divisor: u16) -> Self {
        let registers = peripheral.registers();
        // Disable UART interrupts before anything else touches the block.
        registers.interrupt.set(pac::IER_DISABLE_ALL);

        // Divisor latches share slots with data/interrupt registers: set
        // DLAB, write low/high, then clear DLAB while selecting 8N1.
        let lcr = registers.line_control.get();
        registers.line_control.set(lcr | pac::LCR_DLAB);
        registers.data.set(u32::from(divisor as u8));
        registers.interrupt.set(u32::from((divisor >> 8) as u8));
        registers.line_control.set(pac::LCR_8N1);

        // Enable FIFOs and clear both, then settle on plain FIFO mode with
        // the smallest receive trigger (polling reads single bytes anyway).
        registers
            .fifo
            .set(pac::FCR_FIFO_ENABLE | pac::FCR_RX_FIFO_RESET | pac::FCR_TX_FIFO_RESET);
        registers.fifo.set(pac::FCR_FIFO_ENABLE);

        // Hardware flow control (and its auto mode) stays disabled.
        registers.modem_control.set(0x0);

        Self { peripheral }
    }

    /// Waits for transmitter capacity, then writes one byte.
    pub fn write_byte(&mut self, byte: u8) -> Result<(), UartError> {
        self.wait_for(pac::LSR_THRE)?;
        self.peripheral.registers().data.set(u32::from(byte));
        Ok(())
    }

    /// Writes a byte if the transmitter can accept one right now.
    pub fn try_write_byte(&mut self, byte: u8) -> Result<bool, UartError> {
        let lsr = self.lsr()?;
        if lsr & pac::LSR_THRE == 0 {
            return Ok(false);
        }
        self.peripheral.registers().data.set(u32::from(byte));
        Ok(true)
    }

    /// Reads a byte if one is ready, reporting line errors instead of data.
    pub fn try_read_byte(&mut self) -> Result<Option<u8>, UartError> {
        let lsr = self.lsr()?;
        if lsr & pac::LSR_DATA_READY == 0 {
            return Ok(None);
        }
        let byte = self.peripheral.registers().data.get() as u8;
        Ok(Some(byte))
    }

    /// Waits until holding and shift registers are both empty.
    pub fn flush(&mut self) -> Result<(), UartError> {
        self.wait_for(pac::LSR_THRE | pac::LSR_TRANSMITTER_EMPTY)
    }

    /// Writes a byte slice, byte by byte.
    pub fn write_all(&mut self, bytes: &[u8]) -> Result<(), UartError> {
        for &byte in bytes {
            self.write_byte(byte)?;
        }
        Ok(())
    }

    /// Reads the line status register, folding error bits into typed errors.
    fn lsr(&self) -> Result<u32, UartError> {
        let lsr = self.peripheral.registers().line_status.get();
        // A break reads as an all-zero frame and sets both bits; report the
        // most specific condition first.
        if lsr & pac::LSR_BREAK != 0 {
            return Err(UartError::Break);
        }
        if lsr & pac::LSR_FRAMING != 0 {
            return Err(UartError::Framing);
        }
        if lsr & pac::LSR_PARITY != 0 {
            return Err(UartError::Parity);
        }
        if lsr & pac::LSR_OVERRUN != 0 {
            return Err(UartError::Overrun);
        }
        Ok(lsr)
    }

    /// Polls the line status register until all bits of `mask` are set or
    /// the iteration budget is exhausted.
    fn wait_for(&self, mask: u32) -> Result<(), UartError> {
        for _ in 0..POLL_ITERATION_BUDGET {
            let lsr = self.lsr()?;
            if lsr & mask == mask {
                return Ok(());
            }
        }
        Err(UartError::Timeout)
    }
}

impl<P: Instance> core::fmt::Write for Uart<P> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_all(s.as_bytes()).map_err(|_| core::fmt::Error)
    }
}

/// Rounded 16550 baud divisor and the achievable rate:
/// `divisor = round(clk / (16 * baud))`, `actual = clk / (16 * divisor)`.
///
/// Rejects divisors outside `1..=0xFFFF` and rates missing the error budget.
/// Pure arithmetic so it is unit-testable on the host.
fn divisor_for(uart_clk_hz: u32, baud: u32) -> Result<(u16, u32), UartError> {
    if baud == 0 {
        return Err(UartError::BaudUnsupported { requested: baud });
    }
    let divisor = (u64::from(uart_clk_hz) + 8 * u64::from(baud)) / (16 * u64::from(baud));
    if divisor == 0 || divisor > u64::from(u16::MAX) {
        return Err(UartError::BaudUnsupported { requested: baud });
    }
    let divisor = divisor as u16;
    let actual = u64::from(uart_clk_hz) / (16 * u64::from(divisor));
    let error_ppm = actual.abs_diff(u64::from(baud)) * 1_000_000 / u64::from(baud);
    if error_ppm > MAX_BAUD_ERROR_PPM {
        return Err(UartError::BaudUnsupported { requested: baud });
    }
    Ok((divisor, actual as u32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divisor_at_100mhz_115200() {
        // round(100e6 / (16 * 115200)) = 54; actual = 115740 (0.47% error).
        assert_eq!(divisor_for(100_000_000, 115_200), Ok((54, 115_740)));
    }

    #[test]
    fn divisor_uses_rounding_not_truncation() {
        // 100e6 / (16 * 38400) = 162.76 -> 163; actual = 38343 (0.15% error).
        assert_eq!(divisor_for(100_000_000, 38_400), Ok((163, 38_343)));
    }

    #[test]
    fn baud_beyond_error_budget_is_rejected() {
        // round(100e6 / (16 * 921600)) = 7 -> 892857 actual: 3.1% error.
        assert_eq!(
            divisor_for(100_000_000, 921_600),
            Err(UartError::BaudUnsupported { requested: 921_600 })
        );
    }

    #[test]
    fn divisor_out_of_range_is_rejected() {
        // Requested baud above clk / 16 rounds to divisor 0.
        assert_eq!(
            divisor_for(1_000_000, 115_200),
            Err(UartError::BaudUnsupported { requested: 115_200 })
        );
        // Requested baud far below clk / (16 * 65536) exceeds the latch.
        assert_eq!(
            divisor_for(u32::MAX, 1),
            Err(UartError::BaudUnsupported { requested: 1 })
        );
    }

    #[test]
    fn zero_baud_is_rejected() {
        assert_eq!(
            divisor_for(100_000_000, 0),
            Err(UartError::BaudUnsupported { requested: 0 })
        );
    }
}
