/*
 * ARMv7-A startup for the supported Cyclone V Boot ROM handoff.
 *
 * Incoming (SoC-specific, Cyclone V Boot ROM): r0 = rom_info address,
 * r1 = rom_info length.
 * Outgoing (this runtime's convention): boot_main(r0 = rom_info,
 * r1 = rom_info_len). The conventions are independent; the runtime currently
 * forwards the incoming registers unchanged.
 *
 * Entry sequence:
 *   mask IRQ/FIQ -> select Supervisor mode -> initialize SVC stack
 *   -> configure vector selection and VBAR -> clear BSS -> boot_main
 *
 * Entry-state assumptions (Supervisor mode, IRQ/FIQ masked, cache/MMU in
 * Boot ROM state) are Cyclone V-scoped and must be re-examined per SoC.
 */

.syntax unified
.arm

.section .text._start, "ax", %progbits
.balign 4
.global _start
.type _start, %function

_start:
    /* Mask IRQ and FIQ during startup. */
    cpsid if

    /* Select Supervisor mode and initialize its stack. */
    cps #0x13
    ldr sp, =__stack_top

    /*
     * Select VBAR-based vectors:
     * SCTLR.V, bit 13, selects fixed high vectors when set.
     */
    mrc p15, 0, r2, c1, c0, 0
    bic r2, r2, #0x2000
    mcr p15, 0, r2, c1, c0, 0

    /* Install our 32-byte-aligned exception-vector table. */
    ldr r2, =__vectors_start
    mcr p15, 0, r2, c12, c0, 0

    /* Synchronize subsequent execution with these changes. */
    isb

    /*
     * Preserve incoming r0/r1.
     * Use other registers to clear BSS.
     */
    ldr r2, =__bss_start
    ldr r3, =__bss_end
    mov r4, #0

.Lclear_bss:
    cmp r2, r3
    bhs .Lbss_done
    str r4, [r2], #4
    b .Lclear_bss

.Lbss_done:
    bl boot_main

    /* Defensive fallback if boot_main unexpectedly returns. */
.Lhalt:
    b .Lhalt

.size _start, . - _start
.ltorg