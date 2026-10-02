# j-switch v0.2.0

This release makes downloaded JDKs visible immediately, keeps installations with the same major version distinct, and switches Java in an already configured Windows terminal through a stable junction.

## Changes

- `jsh list` checks registered JDKs, managed installs, `JAVA_HOME`, and configured `scan_dirs`. Use `jsh list --scan` for a broader system search or `jsh list --prune` to remove unavailable registrations.
- Downloaded Temurin JDKs are verified with size and SHA-256 checks, installed in a staging directory, registered immediately, and kept in the configured download cache.
- JRE installations are excluded from JDK discovery. `jsh current` also reports when `java` on `PATH` differs from `JAVA_HOME`.
- Configuration writes are atomic and serialized across jsh processes. `jsh use` restores the previous selection if environment setup fails.
- Shell profile updates on Unix quote paths safely. Windows setup now checks that jsh is installed in a writable location.

## Upgrade note

Install `jsh` in a directory you can write without Administrator rights. Windows needs one elevated `jsh use` to set the system environment variables; reopen the terminal once. Later switches retarget the junction without elevation when the stable `jsh-current/bin` entry precedes other Java entries on `PATH`.

JDK downloads currently use Eclipse Temurin only. `--vendor temurin` and `--vendor adoptium` are aliases for the same source.
