# j-switch (`jsh`)

A small JDK manager for Windows, macOS, and Linux. This repository is a [fork of YiTouch/j-switch](https://github.com/YiTouch/j-switch).

`jsh` discovers existing JDKs, downloads Eclipse Temurin JDKs, keeps installations with the same version separate, and switches the active JDK.

## Install

Download a binary from [Releases](https://github.com/nanocm/j-switch/releases), or build from source with Rust 1.85 or newer (the project uses Rust edition 2024):

```sh
cargo build --release --locked
```

The binary is `target/release/jsh.exe` on Windows and `target/release/jsh` elsewhere. Keep the binary, its `config.json`, `jdks`, `downloads`, and (on Windows) `jsh-current` in one location. On Windows, choose a dedicated directory that your normal account can write without elevation; do not put `jsh.exe` under `Program Files` or the Windows directory. Add that directory to your `PATH` so `jsh` can be called from any terminal.

## Commands

| Command | Effect |
| --- | --- |
| `jsh list` | Refresh managed JDKs, `JAVA_HOME`, and configured scan directories; show registered JDKs. |
| `jsh list --scan` | Also search common system locations. On Windows this walks C: through G: up to five levels deep and may be slow. |
| `jsh list --prune` | Remove registrations whose JDK paths are unavailable. Use this after deleting an installation; disconnected drives can be registered again later. |
| `jsh current` | Show the JDK selected by `JAVA_HOME` (or the saved selection when it is unset) and warn if `java` on `PATH` points elsewhere. |
| `jsh use <version-or-ID>` | Switch to an installed JDK. |
| `jsh search [keyword]` | Search available Eclipse Temurin versions. |
| `jsh download <major>` | Download, verify, install, and register the latest Temurin build for a major version. |

Example:

```sh
jsh list
jsh download 21
jsh list
jsh use 21
jsh current
```

`jsh list` shows each installation's stable ID. `jsh use 21` works when one registered installation matches major version 21. If several do, use the displayed ID; a full version works when it has one match. Old configs keyed only by major version migrate automatically.

Downloads are checked against the size and SHA-256 provided by Adoptium. Verified archives remain in the configured download cache. Installed JDKs live under `jdks` beside the executable and appear in `jsh list` immediately. `--vendor temurin` and `--vendor adoptium` name the same source; other vendors and custom mirrors are not supported.

## Switching in the current terminal

### Windows

The first `jsh use <version-or-ID>` needs an Administrator terminal to configure the **system** `JAVA_HOME` and `PATH`. `jsh` creates a `jsh-current` directory junction beside the executable and puts `jsh-current\bin` first in the system `PATH`. Reopen the terminal once to inherit those values.

Later `jsh use` commands only retarget the junction. A terminal that already uses `jsh-current` for `JAVA_HOME` and resolves `java` from `jsh-current\bin` sees the new JDK immediately. `jsh current` warns when another Java installation precedes it on `PATH`. The `jsh.exe` directory must remain writable by your normal account for later switches.

This setup changes the system Java selection for other terminals and users too. Keep the installation directory under your control and do not grant other users write access to it.

### macOS and Linux

`jsh use` updates the managed lines in your `.zshrc` or `.bashrc`. Run `source ~/.zshrc` or `source ~/.bashrc` in the current shell, as appropriate. New shells read the updated profile automatically.

## Configuration

`config.json` sits beside the executable. Relative `scan_dirs` and `download_dir` paths are resolved from that directory. A scan directory may itself be a JDK root; scanning descends at most five levels. An unavailable registration is shown separately by `jsh list` and can be removed with `jsh list --prune`.

```json
{
  "current_jdk": null,
  "jdks": {},
  "download_dir": "downloads",
  "scan_dirs": ["D:\\Java", "E:\\SDKs"]
}
```

Edit the paths for your machine. `download_dir` stores verified archives; installed JDKs always go under `jdks` beside `jsh`. To keep configs portable when moving the tool, use a relative `download_dir`.

## Development

```sh
cargo test --locked
cargo build --release --locked
```

See [release notes](RELEASE_NOTES.md) for changes in v0.2.0. Licensed under [MIT](LICENSE).
