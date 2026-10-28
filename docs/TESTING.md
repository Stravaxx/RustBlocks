# Testing

Run:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
```

Unit tests cover piece-cell uniqueness and rotation cycles, collisions, line
compaction, seven-bag distribution, seeded game setup, hard drop, hold rules,
pause timing, score/combo saturation, malformed profile handling, WAV output,
and deterministic bounded bytebeat samples.

Tests do not require a display or physical audio device. Interactive screen
navigation, resizing, fullscreen, graphics-driver behavior, actual audio
playback, and perceived audio fidelity require manual platform testing. No
property-testing or benchmark dependency is included yet. A QEMU boot test is
not part of the native test suite.
