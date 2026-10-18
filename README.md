<div align="center">
  <picture>
    <source
      media="(prefers-color-scheme: dark)"
      srcset="https://raw.githubusercontent.com/sidermux/sidermux-assets/main/logos/logo-horizontal.svg"
    >
    <img
      alt="sidermux-hal"
      src="https://raw.githubusercontent.com/sidermux/sidermux-assets/main/logos/logo-horizontal-light.svg"
      width="420"
    >
  </picture>

  <p>
    <strong>Bare-metal HAL for SoC FPGAs: one interface across architectures, SoCs, and boards.</strong>
  </p>

  <p>
    <a href="https://docs.rs/sidermux-hal">Documentation</a>
  </p>
</div>

> [!WARNING]
> **Beta software.** This project is pre-release and breaking changes may
> happen frequently between versions. Pin the exact version you depend on.

---

Cyclone V is the first supported SoC (`armv7a-none-eabi`, dual Cortex-A9).
SoC and architecture support live in HAL; board support lives in BSP; no additional hardware families are claimed as supported.

| Feature | Effect |
|---|---|
| `sidermux-pac/cyclone-v` | Typed Cyclone V peripheral handles and registers |
| `sidermux-hal/cyclone-v` | Cyclone V HAL (implies `arch-armv7a` and the matching PAC) |
| `sidermux-hal/arch-armv7a` | ARMv7-A CPU support, SoC-independent |
| `sidermux-bsp/board-de1-soc` | DE1-SoC board facade and initialization |
| `sidermux-bootloader/board-de1-soc` | Boot image with DE1-SoC handoff data |

`sidermux-hal` builds target-independent with no SoC selected;
`sidermux-bootloader` currently requires the `board-de1-soc` feature.

## Quick start

Build the bootable image:

```sh
cargo build -p sidermux-bootloader --target armv7a-none-eabi --features board-de1-soc
```

## Workspace

```text
sidermux-hal/
├── sidermux-pac/          # typed peripheral handles, register layouts and fields
├── sidermux-hal/          # HAL: reusable SoC and architecture drivers
├── sidermux-bsp/          # board wiring, handoff data and initialization
├── sidermux-rt/           # runtime startup, selected per architecture
└── sidermux-bootloader/   # bootable image; SoC boot profiles live here
```

## Documentation

The full documentation, covering target selection, boot flow layering, and
extension points, lives in the Rust API docs:

- [sidermux-bsp](sidermux-bsp/README.md): board initialization and console facade
- [sidermux-pac](sidermux-pac/README.md): typed register API and ownership contract
- [`sidermux-hal`](https://docs.rs/sidermux-hal): unified HAL surface and
  responsibility split
- [`sidermux-rt`](https://docs.rs/sidermux-rt): runtime startup contracts
- [`sidermux-bootloader`](https://docs.rs/sidermux-bootloader): bootable
  image and boot profiles

Or read them locally:

```sh
cargo doc --workspace --no-deps --features sidermux-bootloader/board-de1-soc --open
```

## Release checks

CI checks formatting, host tests, host and ARM Clippy, documentation, the
release boot image, and builds each crate from its packaged archive.
Reproduce package verification locally with:

```sh
CARGO_PROFILE_DEV_OPT_LEVEL=s cargo package --workspace --all-features --target armv7a-none-eabi
```

Package verification uses development builds; optimization keeps the boot
image within its fixed OCRAM arena. Publish dependencies first:
`sidermux-pac`, `sidermux-hal`, `sidermux-bsp`, and `sidermux-rt`, followed
by `sidermux-bootloader`. Wait for each dependency to become available on
crates.io before publishing its dependents.

## License

MIT or Apache 2.0. Copyright © 2026 sidermux.
