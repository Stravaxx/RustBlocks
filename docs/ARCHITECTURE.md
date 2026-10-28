# Architecture

RustBlocks is one native desktop executable plus an audio diagnostic executable.
The main code is intentionally a small Cargo package rather than a workspace:
there is currently only one shared library and two thin binaries.

## Layers

- `src/game/` owns board coordinates, tetromino geometry, SRS kick transition
  tests, seven-bag randomization, game state, gravity, locking, line clearing,
  score and modes. The engine is serializable and is tested without graphics.
- `src/main.rs` owns macroquad initialization, app screens, input polling,
  frame updates and drawing. The simulation consumes a bounded frame delta;
  game time is not tied to rendering frequency.
- `src/audio/` isolates the bytebeat calculation, WAV tooling and CPAL output.
  The callback owns its sample counter and uses atomic controls.
- `src/persistence.rs` defines version-tolerant settings/profile models and
  user-data paths. Writes use a same-directory temporary file before replace.

## Library choices

- **macroquad 0.4** supplies a lightweight cross-platform 2D window, input and
  primitive rendering without requiring a second GUI framework. Menus are
  drawn in the same canvas; egui would add integration and dependency weight
  without being needed by the current UI.
- **cpal 0.15** provides native output devices on desktop platforms. The audio
  callback is isolated from game/UI code. Failure to initialize audio does not
  prevent the game from starting.
- **serde + TOML** provide readable settings/profile persistence.
- **directories** selects OS-specific per-user data directories.
- **tempfile** provides temporary same-directory files for robust replacement.

Versions are constrained by `Cargo.toml` and pinned by the generated
`Cargo.lock`; licenses and transitive dependencies should be checked when
releasing.

## Boundaries and future work

The current engine stores the active piece and board directly and does not
introduce an event bus or renderer abstraction prematurely. A future controller,
replay engine or bare-metal executable should reuse only the rules crate/module
and introduce platform-specific adapters separately. The current module is not
yet split into an independently published workspace crate.
