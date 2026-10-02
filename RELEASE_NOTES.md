# j-switch v0.2.0

This release makes downloaded JDKs visible immediately, keeps installations with the same major version distinct, and switches Java in an already configured Windows terminal through a stable junction.

## Changes

- `jsh list` checks registered JDKs, managed installs, `JAVA_HOME`, and configured `scan_dirs`. Use `jsh list --scan` for a broader system search or `jsh list --prune` to remove unavailable registrations.
- Downloaded Temurin JDKs are verified with size and SHA-256 checks, installed in a staging directory, registered immediately, and kept in the configured download cache.
- JRE installations are excluded from JDK discovery. `jsh current` also reports when `java` on `PATH` differs from `JAVA_HOME`.
- Configuration writes are atomic and serialized across jsh processes. `jsh use` restores the previous selection if environment setup fails.
- Shell profile updates on Unix quote paths safely. Windows setup now uses user-scoped environment variables and checks that jsh is installed in a writable location.

## Upgrade note

Install `jsh` in a directory you can write without Administrator rights. On Windows, run `jsh use` once and reopen the terminal to inherit the user environment variables. Later switches retarget the junction immediately when `jsh-current/bin` precedes other Java entries on `PATH`. If an earlier development build added `jsh-current/bin` to the system `PATH`, remove that old system entry to complete the move to user-scoped settings.

JDK downloads currently use Eclipse Temurin only. `--vendor temurin` and `--vendor adoptium` are aliases for the same source.
