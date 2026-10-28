# Security policy

RustBlocks is a local desktop game and does not include networking, telemetry,
an updater, or command execution driven by user configuration.

Report suspected vulnerabilities privately to the repository maintainer rather
than publishing exploit details in an issue. Include the affected version,
platform, and a concise reproduction. No security contact address has yet been
configured for this repository.

The project validates persisted configuration using typed deserialization and
does not replace malformed profile data. Dependencies should be reviewed when
updated; `cargo audit` and `cargo deny` are recommended when installed. Neither
tool alone establishes that the application is secure.
