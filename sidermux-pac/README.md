<div align="center">
  <picture>
    <source
      media="(prefers-color-scheme: dark)"
      srcset="https://raw.githubusercontent.com/sidermux/sidermux-assets/main/logos/logo-horizontal.svg"
    >
    <img
      alt="sidermux-pac"
      src="https://raw.githubusercontent.com/sidermux/sidermux-assets/main/logos/logo-horizontal-light.svg"
      width="420"
    >
  </picture>

  <p>
    <strong>Typed peripheral access for SoC FPGAs.</strong>
  </p>

  <p>
    <a href="https://docs.rs/sidermux-pac">Documentation</a>
  </p>
</div>

> [!WARNING]
> **Beta software.** This project is pre-release and breaking changes may
> happen frequently between versions. Pin the exact version you depend on.

---

Typed, `no_std` peripheral access for SoC FPGAs. Cyclone V is the first
supported SoC. Register layouts and fields live here; board configuration
lives in `sidermux-bsp`; reusable drivers live in `sidermux-hal`.

## Quick start

```toml
[dependencies]
sidermux-pac = { version = "0.1", features = ["cyclone-v"] }
```

```rust,no_run
use sidermux_pac::{cyclone_v::{Peripherals, uart}, prelude::*};

let peripherals = Peripherals::take().expect("peripherals already acquired");
let status = peripherals.uart0.line_status.get();
let empty = peripherals.uart0.line_status.is_set(uart::LineStatus::EMPTY);
```

## Register access

The default feature set exposes the common register types and access prelude.
The `cyclone-v` feature exposes `sidermux_pac::cyclone_v`. Other SoCs will have
their own feature and namespace, using the same `tock-registers` access traits:

- `get` / `read` for readable registers and fields;
- `set` / `write` for writable registers and fields;
- `modify` for ordinary read/write registers only.

Command, write-one-to-clear, and aliased registers have distinct read/write
identities, so they do not expose ordinary `modify`. Raw `set` remains available
for exact vendor programming words; it does not validate reserved bits, clock
safety, or the current UART DLAB mode. The UART driver establishes DLAB before
accessing the shared data/interrupt/divisor slots.

## Ownership

Peripheral handles are acquired once, are non-copyable and neither `Send` nor
`Sync`, and keep register references tied to the handle lifetime. Dropping a
handle does not make it available again. `ptr()` and `unsafe steal()` allow
explicit low-level access; the caller then owns the exclusivity obligations.
Taking handles performs no MMIO and does not initialize a peripheral.

Use the singleton from the initialized, privileged **core0** runtime with
core1 parked. On Cyclone V, `take()` checks the core ID, briefly masks IRQ/FIQ,
and restores the original masks around the guard's atomic loads/stores. It
avoids exclusive read-modify-write instructions in early OCRAM; the initial
exclusive-CAS implementation reported an acquisition failure on hardware.
Host tests use compare-and-swap instead.
MMIO must use the correct identity mapping and device/strongly-ordered memory
attributes. This PAC does not configure the MMU, inter-core synchronization,
clocks, resets, or pin routing. It is a reviewed partial register map, not a
complete generated device description.

## References and validation

Register definitions are based on the Cyclone V HPS Technical Reference Manual,
document 683126, register-map chapters 2–6 and UART/timer/watchdog chapters:
https://www.intel.com/programmable/technical-pdfs/683126.pdf

The corresponding vendor U-Boot reference is commit
`b56517573b02f683018e4599e77338e757c6a288`:
https://github.com/altera-fpga/u-boot-socfpga/tree/b56517573b02f683018e4599e77338e757c6a288/arch/arm/mach-socfpga

Run `cargo test -p sidermux-pac --all-features` for register-layout, field,
singleton and compile-fail ownership/access checks. These tests never access
physical MMIO. Layout macro assertions also run at compile time on the ARM target.

The owned-handle HAL migration was verified on DE1-SoC on 2026-10-18:
startup UART banner at 115200 8N1, scan engine idle, all I/O banks thawed,
UART0 reset released, 207 pinmux readbacks matching, and watchdog servicing
active. Runtime captures are kept outside the source tree.

Part of the [sidermux workspace](../README.md).

## License

MIT or Apache 2.0. Copyright © 2026 sidermux.
