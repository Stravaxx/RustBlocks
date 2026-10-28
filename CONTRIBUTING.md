# Contributing

Contributions should preserve the boundary between the platform-independent
rules engine and the desktop runtime. Keep the game logic deterministic and
testable without a window or audio device.

Before opening a change:

1. Format with `cargo fmt --all`.
2. Run `cargo clippy --all-targets --all-features -- -D warnings`.
3. Run `cargo test --workspace`.
4. Update relevant documentation and `docs/IMPLEMENTATION_STATUS.md`.
5. Do not claim a platform was tested unless it was run on that platform.

Keep dependencies focused, avoid `unsafe` unless justified, and include
regression tests for behavior fixes. Do not add telemetry or download executable
content. User data must be validated and preserved when parsing fails.

Pull requests should describe behavior changes, tests performed, and any
platform limitations. The repository currently has no enforced commit message
convention.
