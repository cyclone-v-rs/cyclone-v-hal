/*
 * Cyclone V boot profile: boot image contract.
 *
 * Vectors at the image base; Boot ROM header slot at +0x40; handoff entry
 * slot at +0x4C. The arena and its size limits are profile policy; the
 * physical OCRAM window is declared by the SoC memory script.
 */
ENTRY(boot_entry)

__boot_image_start = ORIGIN(OCRAM);

SECTIONS
{
    .boot_vectors ORIGIN(OCRAM) :
    {
        __vectors_start = .;
        KEEP(*(.boot.vectors))
    } > OCRAM

    .boot_header (ORIGIN(OCRAM) + 0x40) :
    {
        LONG(0);
        LONG(0);
        LONG(0);
    } > OCRAM

    .boot_entry (ORIGIN(OCRAM) + 0x4C) :
    {
        KEEP(*(.boot.entry))
    } > OCRAM
}

ASSERT(SIZEOF(.boot_vectors) > 0,
    "Boot vectors are missing");

ASSERT(SIZEOF(.boot_vectors) <= 0x40,
    "Boot vectors overlap the header");

/* VBAR requires a 32-byte-aligned vector base. */
ASSERT((ADDR(.boot_vectors) & 0x1F) == 0,
    "Boot vectors must be 32-byte aligned for VBAR");

ASSERT(SIZEOF(.boot_entry) == 4,
    "Boot entry must contain one 4-byte ARM branch");

ASSERT(boot_entry == ADDR(.boot_entry),
    "boot_entry must be at the start of the entry section");