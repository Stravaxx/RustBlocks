# Bare-metal feasibility: BIOS Legacy and UEFI

The native game is intentionally separate from any no-OS build. No boot image
or bootloader is currently included or installed. All experimentation should
remain in QEMU or disposable image files; never write a boot sector or EFI
System Partition on a host machine automatically.

## Proposed split

An eventual `baremetal/` package would have a `no_std` core for board, pieces,
scoring and deterministic game state, plus a separate firmware adapter. It
would not reuse macroquad, CPAL, directories or native persistence. A UEFI
application is the lowest-risk first milestone: load as an EFI executable,
draw through GOP, read firmware keyboard input, and keep the game loop in the
application. After validating that core, consider a bootloader/kernel image.

## UEFI

UEFI can load an EFI application and offers GOP framebuffer and console input
protocols. Firmware protocols are available before `ExitBootServices`; after
that transition, firmware boot services cannot be used. A small game can remain
inside boot services and avoid taking over the machine, but input, timing,
resolution and firmware behavior vary. A maintained Rust UEFI crate and its
current target/API should be verified before implementation. This repository
does not currently select or pin such a crate.

## BIOS Legacy

BIOS starts from a boot sector and does not understand Rust executables. A
loader must validate/load the next stage, establish memory and processor state,
and transfer to 32-bit protected mode or 64-bit long mode. Disk geometry,
boot-sector size, memory maps and legacy video modes add substantial
compatibility concerns. A bootloader crate or existing project is preferable
to hand-writing an MBR. BIOS support should follow—not precede—a UEFI prototype.

## Hardware and features

QEMU is appropriate for disposable disk images and reproducible boot tests;
it is not installed/available in the current environment. GOP offers a
framebuffer, not a GPU acceleration API. Firmware keyboard protocols may cover
basic keys but not arbitrary USB/controller behavior once boot services end.
Audio is especially uncertain: native audio drivers and CPAL are unavailable
without an OS; PC speaker output may be possible on selected BIOS/VM targets,
but is low fidelity and not portable. The bytebeat formula can be reused as a
pure sample generator, but would need a hardware-specific output path.

## Implementation sequence and effort

1. Extract the tested rules into a dependency-light core crate.
2. Prototype UEFI GOP rendering/input and game loop in QEMU.
3. Verify UEFI image creation and failure recovery entirely in virtual media.
4. Decide whether BIOS is worth a separate loader/kernel integration.
5. Treat audio, persistence, controller support and physical-machine testing as
   independent, optional projects.

The UEFI demo is a medium/high effort prototype; BIOS Legacy boot and reliable
hardware support are high effort with recurring maintenance across firmware
implementations. Neither is required for the native release, and there is no
bare-metal prototype in this delivery.
