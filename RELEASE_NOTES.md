# j-switch v0.4.0

## Install from GitHub

`install.ps1` and `install.sh` download the matching Windows x64, Linux x64, or macOS x64/ARM64 release, verify its SHA-256 checksum, and install `jsh` into a user-writable directory. Both scripts accept a custom directory. If that directory is missing from `PATH`, they ask before adding it to the user's environment or shell profile. The scripts and their checksums are included in this release.

## Configuration filename

The configuration beside `jsh` is now named `jsh_config.json`, avoiding collisions with an unrelated `config.json` in the executable directory. On first run, an existing jsh `config.json` is imported into the new file; the old file remains in place. The new file takes precedence once it exists.

See the [English](https://github.com/nanocm/j-switch/blob/v0.4.0/README.md) and [中文](https://github.com/nanocm/j-switch/blob/v0.4.0/README.zh-CN.md) guides for installation commands and directory settings.
