<div align="center">
  <picture>
    <source
      media="(prefers-color-scheme: dark)"
      srcset="https://raw.githubusercontent.com/sidermux/sidermux-assets/main/logos/logo-horizontal.svg"
    >
    <img
      alt="sidermux-bsp"
      src="https://raw.githubusercontent.com/sidermux/sidermux-assets/main/logos/logo-horizontal-light.svg"
      width="420"
    >
  </picture>

  <p>
    <strong>Board configuration and initialization for sidermux targets.</strong>
  </p>

  <p>
    <a href="https://docs.rs/sidermux-bsp">Documentation</a>
  </p>
</div>

> [!WARNING]
> **Beta software.** This project is pre-release and breaking changes may
> happen frequently between versions. Pin the exact version you depend on.

---

Board wiring, vendor handoff data, and initialization for sidermux. This
`no_std` crate has no default features. Enable `board-de1-soc` for DE1-SoC.

The dependency chain is bootloader → BSP → HAL → PAC. UART register handling
and the reusable UART driver remain in PAC and HAL; the BSP chooses UART0,
its pins, input clock, baud rate, and board initialization order.

## Quick start

```toml
[dependencies]
sidermux-bsp = { version = "0.1", features = ["board-de1-soc"] }
```

```rust,no_run
use sidermux_bsp::de1_soc::Board;
let mut board = Board::take().expect("board already acquired or not on core 0");
let reset_cause = board.reset_cause();
let console = board.initialize_console().expect("board initialization failed");
```

## Initialization

`Board::take` services inherited watchdogs and captures reset causes. Console
initialization follows the vendor sequence: freeze, reset preparation, clocks,
IOCSR, pinmux, UART reset release, thaw, console. Warm-reset I/O handling
brackets IOCSR and pinmux programming. Successful calls reuse the console;
failed calls return the cached error and require reset before retrying.
`service_watchdogs` remains available after failure and during idle.

## Handoff and validation

The vendor source revision, original header hashes, and exact scan bit counts
are documented in [HANDOFF.md](src/de1_soc/HANDOFF.md). The Rust
transcription retains its vendor provenance and BSD notice. Run the BSP checks with:

```sh
cargo test -p sidermux-bsp --all-features
```

The bootloader retains the Boot ROM entry assembly, image layout, banner,
application failure handling and debugger-visible `BOOT_STATUS`.

Hardware regression passed on DE1-SoC on 2026-10-18: startup banner received
at 115200, no boot error, scan manager idle/FIFO empty, required banks thawed,
UART0 reset released, all 207 pinmux entries matching, and watchdog0 serviced.
Clock rates remain register-derived; UART bit time was not measured with an
instrument. IOCSR and thawing have not been isolated as independent causes
of the original TX failure.

Part of the [sidermux workspace](../README.md).

## License

MIT or Apache 2.0. Copyright © 2026 sidermux.

Vendor-derived handoff data carries the notice documented in
[HANDOFF.md](src/de1_soc/HANDOFF.md).
