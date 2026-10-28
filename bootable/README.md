# RustBlocks live image

This creates a hybrid BIOS/UEFI Debian live ISO that starts XFCE and launches
RustBlocks automatically. It is a Linux live system, not a no-OS bootloader.
The native game and its rules remain unchanged.

## Build

On Debian or Ubuntu Linux, install Rust stable, `live-build`, `xorriso`,
`debootstrap`, `squashfs-tools`, and the normal Rust build prerequisites
(including ALSA development headers). Then run:

```sh
./bootable/build-iso.sh
```

The ISO and SHA-256 checksum are written to `dist/`. The first build needs
network access to download Debian packages and Rust crates. The resulting
image targets amd64 PCs and VMs; firmware menus may need Secure Boot disabled.
Try it in a disposable QEMU VM before writing it to USB. The build script never
writes to a physical disk.

## Persistence and sharing

The live system stores profiles and records in its writable session. They are
lost after shutdown unless the user configures persistent storage or exports
records to another drive. Use a four-letter A–Z player tag in **Player profile**.

Records can be exchanged as CSV files on a USB drive:

```sh
rustblocks --export-scores /media/player/USB/rustblocks-scores.csv
rustblocks --import-scores /media/player/USB/rustblocks-scores.csv
```

Import merges records by player and mode, keeping the highest score. This is
offline file sharing; the project does not include an online server or live
network multiplayer.
