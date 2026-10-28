# Release procedure

## Automated GitHub release

To publish a release:

1. Update `package.version` in `Cargo.toml` and the root `rustblocks` entry in
   `Cargo.lock` with the same SemVer version.
2. Add a release at the top of `CHANGELOG.md`, using the exact heading
   `## X.Y.Z - YYYY-MM-DD`, and include non-empty release notes under it.
3. Merge/push those changes to the repository's default branch. The
   `Release packages` workflow checks that the changelog version matches
   `Cargo.toml`, builds all six native targets, then creates the `vX.Y.Z`
   GitHub release and uploads each package with a SHA-256 file. The release
   notes combine the changelog entry with the pushed commit's SHA, author, date
   and full message.

The workflow produces ZIP archives for Windows x86_64 and ARM64, and tar.gz
archives for Linux x86_64/ARM64 and macOS Intel/Apple Silicon. Each archive
contains both RustBlocks binaries, README, license and changelog. It does not
sign/notarize binaries or build native installers. Linux packages are built on
Ubuntu and require the host's graphics and ALSA runtime libraries.

The workflow only publishes changes made on the default branch. A failed
version check or build prevents publication. The same workflow can be started
manually from the default branch to retry a release; an already-created tag
must be handled intentionally rather than overwritten.
