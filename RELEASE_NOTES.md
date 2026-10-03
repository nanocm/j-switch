# j-switch v0.2.0

Manage installed JDKs and download Eclipse Temurin builds with `jsh`.

## Changes

- `jsh download` verifies archive size and SHA-256, installs the JDK, and registers it immediately. Windows JDK 8 packages are recognized correctly after extraction.
- Installations with the same Java version keep separate IDs, which `jsh use` accepts when a version is ambiguous.
- Numeric `jsh search` queries match the exact major version, so `jsh search 8` does not include JDK 18.
- `jsh list` checks managed installs, `JAVA_HOME`, and configured `scan_dirs` by default. Use `--scan` for a broader system search or `--prune` to remove unavailable registrations.
- Windows uses a user-scoped `JAVA_HOME` and a stable directory junction. Bash and zsh use a stable symbolic link, so later switches can take effect in an open terminal.

After the first `jsh use`, reopen the terminal once on Windows or source the updated shell profile once on macOS or Linux. The active shell must find Java through `jsh-current/bin` for later switches to take effect immediately.

Downloads currently use Eclipse Temurin. `--vendor temurin` and `--vendor adoptium` name the same source.
