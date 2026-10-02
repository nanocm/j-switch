# j-switch v0.2.1

This release fixes a real-JDK download failure in v0.2.0. Adoptium's `semver` field can differ from the version reported by `java -version` (for example, `21.0.12+101.0.LTS` versus `21.0.12.1`). `jsh download` now validates against Adoptium's `openjdk_version` field while retaining size and SHA-256 archive checks. The v0.2.0 release was withdrawn.

## Validation

- The repository owner confirmed Windows operation; Windows CI tests and release builds passed.
- Hosted Linux x64, macOS Intel, and macOS ARM64 runners each downloaded a real Temurin 21 JDK, listed it, switched from Java 17 to 21 in one open bash shell, and verified `java`, `javac`, and `jsh current`.
- Release publication is now gated on those real-JDK smoke tests. The release archives are checked for SHA-256 integrity and CPU architecture.

## Changes since the original project

- Downloaded JDKs are registered immediately; installations sharing a major or full version keep distinct IDs.
- `jsh list` scans managed installs, `JAVA_HOME`, and configured `scan_dirs` by default. Broader system scanning requires `--scan`; unavailable registrations can be removed with `--prune`.
- JREs are excluded from JDK discovery. Configuration writes are atomic and serialized across processes.
- Windows uses user-scoped environment variables and a stable junction. On bash and zsh, a stable symbolic link lets subsequent switches take effect in an initialized open terminal.

On Windows, reopen the terminal once after the first `jsh use`; on bash or zsh, source the updated profile once. Later switches take effect in an open terminal when `jsh-current/bin` is the Java path used by that shell. Downloads use Eclipse Temurin; `--vendor temurin` and `--vendor adoptium` name the same source.
