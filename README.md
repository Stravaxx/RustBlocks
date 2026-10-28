# RustBlocks

RustBlocks is a native Rust falling-block puzzle game. It runs as a desktop
application (not a web wrapper), with a deterministic rules engine, keyboard
menus, six selectable play modes, persistent settings and records, and an
integrated procedural bytebeat soundtrack.

## Requirements

- Rust stable and Cargo (edition 2024).
- A desktop graphics environment supported by macroquad.
- For live music, a working audio output device and the platform audio
  development libraries needed by CPAL. Audio is optional; the game logs a
  diagnostic and continues without it if device initialization fails.

## Build and run

```sh
cargo run --release --bin rustblocks
```

Create a distributable executable with `cargo build --release`. Build
dependencies and platform caveats are documented in [docs/BUILD.md](docs/BUILD.md).

## Tests and checks

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
```

See [docs/TESTING.md](docs/TESTING.md) for what these checks cover and what
still requires interactive/manual validation.

## Controls

| Action | Key |
|---|---|
| Move | Left / Right or A / D |
| Soft drop | Down or S |
| Hard drop | Space |
| Rotate clockwise | Up or X |
| Rotate counter-clockwise | Z |
| Hold | C or Left Shift |
| Pause / resume | Escape or P |
| Navigate menus | Up / Down, Enter |

## Audio diagnostic

Generate a WAV file from the same Rust bytebeat generator used by the live
audio callback (mono PCM16, 44.1 kHz by default):

```sh
cargo run --release --bin rustblocks-audio -- rustblocks-test.wav 10 44100
```

Arguments are output path, seconds (1–600), and sample rate (8,000–192,000).
Bytebeat semantics and known limitations are in [docs/AUDIO.md](docs/AUDIO.md).

## Configuration and data

Profile data is stored in the operating system's per-user data directory,
under the RustBlocks application directory. On macOS this resolves through
`ProjectDirs` to the user's Application Support data path; Linux and Windows
use their standard per-user data locations. Invalid TOML is reported and
defaults are used without overwriting the original file.

## Player profiles and score sharing

Select **Player profile** to choose a four-letter A–Z tag. Personal bests are
stored per profile and game mode. Records can be exchanged as CSV for offline
sharing on USB; see [bootable/README.md](bootable/README.md).

## Bootable live ISO

On Debian or Ubuntu Linux, `./bootable/build-iso.sh` builds a hybrid amd64
BIOS/UEFI live ISO that launches the game automatically. The live session is
ephemeral unless persistent storage is configured. This is a Linux live image,
not a bare-metal Rust kernel; it currently has no online matchmaking or live
network multiplayer. See [bootable/README.md](bootable/README.md).

## Architecture and status

- Rules and deterministic seven-bag generator: `src/game/`
- Macroquad desktop UI and game loop: `src/main.rs`
- CPAL callback, Dollchan bytebeat and WAV writer: `src/audio/`
- Profile persistence: `src/persistence.rs`

Read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md),
[docs/GAMEPLAY.md](docs/GAMEPLAY.md), and the honest feature matrix in
[docs/IMPLEMENTATION_STATUS.md](docs/IMPLEMENTATION_STATUS.md). Bare-metal
boot remains a separate future experiment; see
[docs/BAREMETAL_FEASIBILITY.md](docs/BAREMETAL_FEASIBILITY.md).

## Licensing

Application code is offered under MIT; see [LICENSE](LICENSE). Dependencies
retain their own licenses and notices. No third-party art or recorded music is
bundled. The bytebeat expression was supplied in the project specification;
its original authorship/licensing provenance was not provided, so this project
does not make a separate ownership or license claim about that expression.
