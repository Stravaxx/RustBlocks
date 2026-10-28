# Implementation status

Status reflects repository code and checks, not aspiration. "Implemented" means
code exists; "tested" means a corresponding automated check passed; platform
validation requires actual execution on that platform.

| Feature | State | Files / tests | Remaining limitations |
|---|---|---|---|
| Native desktop setup | Implemented | `Cargo.toml`, `src/main.rs` | Manual graphics run not yet performed |
| Board, pieces, collisions, lock and line clear | Implemented | `src/game/`; game unit tests | No animated line-clear transition |
| Seven-bag and seeded engine | Tested | `src/game/rng.rs`, game setup tests | RNG is small xorshift, not cryptographic |
| SRS rotation kicks | Tested | `src/game/piece.rs`, engine wall/floor kick tests | Broader transition matrix still desirable |
| Scoring, levels, combos | Tested | `src/game/scoring.rs` | No T-spin/back-to-back |
| Classic, marathon, sprint, ultra, zen | Implemented | `src/game/`, mode checks | Custom uses standard defaults; ultra duration not manually validated |
| Menus, pause, help, settings | Implemented | `src/main.rs` | Mouse works for primary menus; key remapping/accessibility options missing |
| Adaptive visual layout | In progress | `src/main.rs` | Tiny-window/reduced-motion/a11y testing outstanding |
| Settings and records persistence | Tested | `src/persistence.rs` | No in-progress resume or delete confirmation; schema v1 only |
| Four-letter local player profiles | Implemented | `src/main.rs`, `src/persistence.rs` | Profiles are local; no simultaneous split-screen match |
| Offline score sharing | Tested | `src/persistence.rs`, score import/export commands | CSV via removable storage; no online service |
| Bytebeat port, WAV diagnostic and generated effects | Tested | `src/audio/` | Exact Dollchan runtime clock convention remains an explicit interpretation |
| CPAL live output | Implemented | `src/audio/mod.rs` | Physical playback not tested on this machine |
| Linux / macOS / Windows builds | Not yet validated | `.github/workflows/ci.yml` | CI has not run in this environment |
| Bootable live ISO recipe | Implemented, not built here | `bootable/build-iso.sh` | Requires Debian/Ubuntu amd64 build host; no ISO or QEMU validation from this macOS ARM environment |
| Bare-metal feasibility | Documented | `docs/BAREMETAL_FEASIBILITY.md` | No no-OS prototype |
| Distribution packages/signing | Not started | `docs/RELEASE.md` | Only native Cargo builds are configured |

## Validation log

- Rust toolchain available: rustc 1.98.1, Cargo 1.98.1; host target
  `aarch64-apple-darwin`.
- Initial compile caught and fixed a level conversion type error.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --workspace --locked`: passed; 25 tests.
- `cargo build --release --locked --bins`: passed for the available macOS host.
- The audio diagnostic generated a one-second mono PCM16/44.1 kHz WAV;
  `file` confirmed a valid RIFF/WAVE header. The temporary file was removed.
- The environment has no Linux/Windows targets installed and no QEMU binary;
  cross-platform runtime validation is delegated to the checked-in CI matrix.
- The graphical game and live audio output have not been manually exercised;
  therefore native compilation and automated tests do not establish that every
  interactive feature works on physical devices.
