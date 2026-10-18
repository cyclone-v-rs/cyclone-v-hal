<div align="center">
  <picture>
    <source
      media="(prefers-color-scheme: dark)"
      srcset="https://raw.githubusercontent.com/sidermux/sidermux-assets/main/logos/logo-horizontal.svg"
    >
    <img
      alt="sidermux-bootloader"
      src="https://raw.githubusercontent.com/sidermux/sidermux-assets/main/logos/logo-horizontal-light.svg"
      width="420"
    >
  </picture>

  <p>
    <strong>Bootable images for sidermux targets.</strong>
  </p>

  <p>
    <a href="https://docs.rs/sidermux-bootloader">Documentation</a>
  </p>
</div>

> [!WARNING]
> **Beta software.** This project is pre-release and breaking changes may
> happen frequently between versions. Pin the exact version you depend on.

---

Layering: architecture startup (`sidermux-rt`), SoC handoff adapter
(`boot_main`), common boot logic. `boot_main`'s signature mirrors the
Cyclone V Boot ROM handoff; it is the adapter, not a portable ABI.

## Quick start

```sh
cargo build -p sidermux-bootloader --target armv7a-none-eabi --features board-de1-soc
```

## Board selection

The bootloader requires `board-de1-soc`, which enables the DE1-SoC BSP
and ARMv7-A runtime.

| Feature | Effect |
|---|---|
| `board-de1-soc` | Selects DE1-SoC board data and enables `cyclone-v` |
| `cyclone-v` | Selects the boot profile and `sidermux-rt/armv7a` |

## Boot profiles

```text
ld/linkall.x              # application layout, generic over staged names
ld/profiles/cyclone-v/    # boot contract: vectors, header slot, entry
                          # slot, size limits, trailer reservation
src/profiles/cyclone-v/   # Boot ROM entry assembly
```

Profiles stage under uniform names (`sidermux-boot.x`,
`sidermux-boot-reserve.x`); the name is a build interface, the contents are
per-SoC. The Cyclone V profile loads at `0xFFFF0000` with vectors at the
image base, a Boot ROM header slot at `+0x40`, the handoff entry at
`+0x4C`, 16-byte packaging, and a 60 KiB arena.

DE1-SoC initialization lives in `sidermux-bsp/src/de1_soc.rs`. It services
inherited watchdogs, freezes I/O, configures clocks and the matching IOCSR
and pinmux handoff, releases UART0 reset, thaws I/O and initializes the
console. The bootloader prints one startup banner, then idles while servicing
watchdogs through an owned BSP `Board`. The BSP acquires peripheral handles
once; UART0 moves into its HAL driver. `BOOT_STATUS` preserves the initial reset cause and any typed
initialization error for debugger inspection. Unexpected panics record a failure
without stealing peripheral handles; an inherited watchdog can then reset the system.

The DE1-SoC reference data and provenance live in
`sidermux-bsp/src/de1_soc/`. Run the BSP checks with
`cargo test -p sidermux-bsp --all-features`. The BSP boot path was verified on DE1-SoC on 2026-10-18 at 115200
baud: startup banner received, no recorded boot error, scan busy/FIFO status
clear, required banks thawed, UART0 reset released, all 207 pinmux entries
matching, and watchdog0 serviced. IOCSR and thawing have not been isolated
as independent causes of the original TX failure. Clock rates remain
register-derived; no instrument-based bit-time measurement was made.

Part of the [sidermux-hal](https://github.com/sidermux/sidermux-hal)
workspace, with [`sidermux-hal`](https://docs.rs/sidermux-hal) and
[`sidermux-rt`](https://docs.rs/sidermux-rt).

## License

MIT or Apache 2.0. Copyright © 2026 sidermux.
