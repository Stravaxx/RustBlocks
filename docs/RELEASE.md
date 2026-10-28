# Release procedure

1. Review the feature and platform state in `IMPLEMENTATION_STATUS.md`.
2. Run formatting, Clippy and the full test suite on a clean checkout.
3. Build natively on Linux, macOS and Windows using the CI matrix.
4. Review the dependency licenses and advisories, including transitive crates.
5. Build each platform with `cargo build --release`, archive the executable
   together with README, license and third-party notices, then calculate
   SHA-256 checksums with the platform's standard tool (`shasum -a 256` on
   macOS, `sha256sum` on Linux, `Get-FileHash -Algorithm SHA256` on Windows).
6. Publish checksums and platform-specific limitations alongside each asset.
7. Sign/notarize macOS builds and sign Windows packages only when publisher
   credentials and a separate release workflow have been configured.

This repository currently has CI builds but no packaging/signing workflow.
There are no release artifacts in the initial implementation.
