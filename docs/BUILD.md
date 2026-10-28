# Build guide

Install the current stable Rust toolchain for the target platform. Rust edition
2024 is used. Cargo downloads crates listed in `Cargo.lock`.

## macOS

```sh
cargo build --release
cargo run --release --bin rustblocks
```

CPAL uses the macOS audio stack; an output device is required for music, but
not for the rules/tests. Native builds use the host architecture (Apple Silicon
or Intel). Cross-architecture builds require the corresponding Rust target and
compatible native dependencies. No signing or notarization has been performed;
distribution outside local development requires the publisher's Apple signing
identity and notarization workflow.

## Linux

Install a C compiler and the development packages required by CPAL/ALSA and
the selected macroquad/miniquad graphics backend. On Debian/Ubuntu, `pkg-config`
and ALSA development headers are commonly required. X11/Wayland behavior depends
on the window backend and installed system libraries; test on the intended
desktop. PipeWire/PulseAudio compatibility depends on the system's ALSA setup.

```sh
cargo build --release
./target/release/rustblocks
```

The current project does not build an AppImage, Flatpak, DEB or RPM package.

## Windows

Install Rust MSVC and the Visual Studio C++ Build Tools, then:

```powershell
cargo build --release
.\target\release\rustblocks.exe
```

The window/audio backends rely on available graphics and audio drivers. The
current project does not generate an installer or signed package.

## Cross compilation

Add a Rust target with `rustup target add <triple>` only after installing the
target's required native linker and libraries. CPAL and window-system native
dependencies make cross-compilation host-dependent. The CI matrix builds on
native Linux, macOS and Windows runners instead.
