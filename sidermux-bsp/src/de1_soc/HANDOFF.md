# DE1-SoC QTS handoff

The Rust configuration is transcribed from the three BSD-3-Clause headers in
`altera-fpga/u-boot-socfpga`, commit
`b56517573b02f683018e4599e77338e757c6a288` (`socfpga_v2022.10`),
`board/terasic/de1-soc/qts/`:

https://github.com/altera-fpga/u-boot-socfpga/tree/b56517573b02f683018e4599e77338e757c6a288/board/terasic/de1-soc/qts

The upstream headers do not identify an original Quartus project revision.

| File | SHA-256 |
|---|---|
| iocsr_config.h | 0749511bc57190009cbb3239995c4a48e161e73bca554edadc69ef9d80022178 |
| pinmux_config.h | 74c070c3f0cd451cf4168069e940a2fcc3154266eb0ab26c4f31d11dd14bed28 |
| pll_config.h | 63a20f79e02795641fd52ff76e96b42c1587b27f56945f27a64b00330b634219 |

`handoff.rs` is the Rust transcription. Chains 0–3 contain exactly
764, 1719, 955, and 16766 valid bits (24, 54, 30, and 524 storage words).
The final packet lengths are 124, 55, 59, and 126 bits. The scan command
carries the valid bit count even when a FIFO transfer includes padding.
All 207 pinmux registers, including UART0USEFPGA at index 189, are applied
and read back. Clock dividers, counters, sources and PLL settings come from
the matching pll_config.h; this selects peripheral PLL / 5 / 2 for L4 SP,
register-derived 100 MHz with the board's specified 25 MHz oscillator.

The SoC implementation follows the register protocol and sequencing described
by the same vendor tree's `arch/arm/mach-socfpga/{spl_gen5.c,
clock_manager_gen5.c,scan_manager.c,freeze_controller.c,system_manager_gen5.c}`.
It services both inherited watchdogs instead of relying on a CR.EN clear
or resetting an active watchdog. It only releases peripherals needed for
this console milestone; it does not enable FPGA bridges or initialize DDR.

The original C headers are not shipped in this repository. The transcription
was compared against the pinned headers before hardware validation. For
configuration changes, retrieve the headers at the pinned revision, verify
the SHA-256 values above, and compare the Rust values against that source.

The BSD license and copyright notice for the transcribed data are retained in
`LICENSE-BSD-3-Clause`.
