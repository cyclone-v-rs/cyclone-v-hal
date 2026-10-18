//! Cyclone V IOCSR command stream. Status `ACTIVE[31]` and `WFIFOCNT[30:28]`
//! must both be zero. Each TDI packet declares its exact valid bit count.
use sidermux_pac::prelude::*;

use super::{early_boot::BootSupport, pac::ScanManager};
const BUSY: u32 = 0xf0000000;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanError {
    InvalidChain,
    InvalidLength,
    PortInUse,
    Timeout { status: u32 },
}
pub struct Chain {
    pub bits: usize,
    pub words: &'static [u32],
}
pub fn status(manager: &ScanManager) -> u32 {
    manager.stat.get()
}
trait Engine {
    fn read(&mut self, offset: usize) -> u32;
    fn write(&mut self, offset: usize, value: u32);
    fn service(&mut self);
}
struct Mmio<'a> {
    manager: &'a mut ScanManager,
    support: &'a mut BootSupport,
}
impl Engine for Mmio<'_> {
    fn read(&mut self, offset: usize) -> u32 {
        match offset {
            0 => self.manager.stat.get(),
            4 => self.manager.enable.get(),
            _ => unreachable!("scan read slot"),
        }
    }
    fn write(&mut self, offset: usize, value: u32) {
        match offset {
            4 => self.manager.enable.set(value),
            0x10 => self.manager.single.set(value),
            0x14 => self.manager.double.set(value),
            0x1c => self.manager.quad.set(value),
            _ => unreachable!("scan write slot"),
        }
    }
    fn service(&mut self) {
        self.support.service_watchdogs();
    }
}
fn idle(engine: &mut impl Engine) -> Result<(), ScanError> {
    for i in 0..100_000 {
        if engine.read(0) & BUSY == 0 {
            return Ok(());
        }
        if i % 256 == 0 {
            engine.service();
        }
    }
    Err(ScanError::Timeout {
        status: engine.read(0),
    })
}
// FIFO apertures accept a word write but enqueue 1, 2, or 4 bytes.
fn packet(words: &[u32], bits: usize, mut emit: impl FnMut(usize, u32)) {
    emit(0x14, (((bits - 1) as u32) << 8) | 0x80);
    for &word in &words[..bits / 32] {
        emit(0x1c, word);
    }
    let tail = bits % 32;
    if tail > 24 {
        emit(0x1c, words[bits / 32]);
    } else if tail != 0 {
        for shift in (0..tail).step_by(8) {
            emit(0x10, (words[bits / 32] >> shift) & 0xff);
        }
    }
}
/// Caller freezes the banks and releases SCANMGR reset first. For HIO chain3
/// the system manager must also deassert DLLRST before calling this method.
pub fn configure(
    manager: &mut ScanManager,
    support: &mut BootSupport,
    id: usize,
    chain: &Chain,
) -> Result<(), ScanError> {
    configure_with(&mut Mmio { manager, support }, id, chain)
}
fn configure_with(engine: &mut impl Engine, id: usize, chain: &Chain) -> Result<(), ScanError> {
    if id > 3 {
        return Err(ScanError::InvalidChain);
    }
    if chain.bits == 0 || chain.words.len() != chain.bits.div_ceil(32) {
        return Err(ScanError::InvalidLength);
    }
    idle(engine)?;
    if engine.read(4) != 0 {
        return Err(ScanError::PortInUse);
    }
    engine.write(4, 1 << id);
    let result = (|| {
        let mut sent = 0;
        while sent < chain.bits {
            engine.service();
            let bits = (chain.bits - sent).min(128);
            packet(&chain.words[sent / 32..], bits, |off, value| {
                engine.write(off, value)
            });
            idle(engine)?;
            sent += bits;
        }
        Ok(())
    })();
    engine.write(4, 0);
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_partial_packets() {
        // All DE1-SoC chain tails, plus FIFO aperture boundaries.
        for bits in [1, 8, 9, 24, 25, 30, 55, 59, 124, 126, 128] {
            let mut bytes = 0;
            let mut header = 0;
            packet(&[0x12345678; 4], bits, |offset, value| {
                if offset == 0x14 {
                    header = value;
                } else {
                    bytes += if offset == 0x1c { 4 } else { 1 };
                }
            });
            assert_eq!(header, (((bits - 1) as u32) << 8) | 0x80);
            assert_eq!(bytes, bits.div_ceil(8));
        }
    }

    extern crate std;
    #[derive(Default)]
    struct Fake {
        writes: std::vec::Vec<(usize, u32)>,
        stuck: u32,
        enabled: bool,
        reads: usize,
    }
    impl Engine for Fake {
        fn read(&mut self, _: usize) -> u32 {
            self.reads += 1;
            if self.enabled { self.stuck } else { 0 }
        }
        fn write(&mut self, off: usize, v: u32) {
            self.writes.push((off, v));
            if off == 4 {
                self.enabled = v != 0;
            }
        }
        fn service(&mut self) {}
    }
    #[test]
    fn exact_board_lengths_and_word_order() {
        for (id, bits) in [764usize, 1719, 955, 16766].into_iter().enumerate() {
            let mut f = Fake::default();
            let chain = Chain {
                bits,
                words: &[0x12345678; 524][..bits.div_ceil(32)],
            };
            configure_with(&mut f, id, &chain).unwrap();
            assert_eq!(f.writes.first(), Some(&(4, 1 << id)));
            assert_eq!(f.writes.last(), Some(&(4, 0)));
            let headers: std::vec::Vec<_> = f
                .writes
                .iter()
                .filter(|(o, _)| *o == 0x14)
                .map(|(_, v)| ((v >> 8) + 1) as usize)
                .collect();
            assert_eq!(headers.iter().sum::<usize>(), bits);
            assert_eq!(headers.len(), bits.div_ceil(128));
            assert_eq!(*headers.last().unwrap(), bits % 128);
            assert!(
                f.writes
                    .iter()
                    .filter(|(o, _)| *o == 0x1c)
                    .all(|(_, v)| *v == 0x12345678)
            );
        }
    }
    #[test]
    fn active_and_fifo_nonempty_both_timeout_and_disable_chain() {
        for stuck in [0x80000000, 0x10000000] {
            let mut f = Fake {
                stuck,
                ..Fake::default()
            };
            let result = configure_with(
                &mut f,
                0,
                &Chain {
                    bits: 1,
                    words: &[0],
                },
            );
            assert_eq!(result, Err(ScanError::Timeout { status: stuck }));
            assert_eq!(f.writes.last(), Some(&(4, 0)));
            assert!(!f.enabled);
            assert!(f.reads < 100_010);
        }
    }
    #[test]
    fn invalid_input_never_touches_hardware() {
        let mut f = Fake::default();
        assert_eq!(
            configure_with(
                &mut f,
                4,
                &Chain {
                    bits: 1,
                    words: &[0]
                }
            ),
            Err(ScanError::InvalidChain)
        );
        assert_eq!(
            configure_with(
                &mut f,
                0,
                &Chain {
                    bits: 33,
                    words: &[0]
                }
            ),
            Err(ScanError::InvalidLength)
        );
        assert_eq!(f.reads, 0);
        assert!(f.writes.is_empty());
    }
}
