# j-switch (`jsh`)

A JDK manager with builds for Windows, macOS, and Linux. This repository is a [fork of YiTouch/j-switch](https://github.com/YiTouch/j-switch).

`jsh` discovers existing JDKs, downloads Eclipse Temurin JDKs, keeps installations with the same version separate, and switches the active JDK.

## Validation status

Windows operation has been confirmed by the repository owner. For Linux and macOS, CI has compiled and run unit tests on hosted runners (including a shell test with simulated Java executables), and the release archives have passed checksum and architecture checks. At the v0.2.0 release, a full `jsh download` → `jsh list` → `jsh use` run with a real JDK had not been verified on Linux or macOS. These builds should be treated as needing real-world validation.

## Install

Download a binary from [Releases](https://github.com/nanocm/j-switch/releases) (Windows x64, Linux x64, or macOS ARM64/x64), or build from source with Rust 1.88 or newer (the locked dependencies require it):

```sh
cargo build --release --locked
```

The binary is `target/release/jsh.exe` on Windows and `target/release/jsh` elsewhere. Keep the binary, its `config.json`, `jdks`, `downloads`, and `jsh-current` in one location. Choose a dedicated directory that your normal account can write without elevation; on Windows, do not put `jsh.exe` under `Program Files` or the Windows directory. Add that directory to your `PATH` so `jsh` can be called from any terminal.

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

The first `jsh use <version-or-ID>` configures your **user** `JAVA_HOME` and `PATH`; Administrator rights are not needed. `jsh` creates a `jsh-current` directory junction beside the executable and adds `jsh-current\bin` to the beginning of your user `PATH`. Reopen the terminal once to inherit those values.

Later `jsh use` commands only retarget the junction. A terminal that already uses `jsh-current` for `JAVA_HOME` and resolves `java` from `jsh-current\bin` sees the new JDK immediately. Windows can place system `PATH` entries before user entries; if another Java installation comes first, `jsh current` reports its location so you can adjust that entry. The `jsh.exe` directory must remain writable by your normal account for later switches.

The setting applies to your Windows account. Keep the installation directory under your control and do not grant other users write access to it.

### macOS and Linux

`jsh use` creates a `jsh-current` symbolic link beside the executable and updates the managed lines in your `.zshrc` or `.bashrc` to use it. Run `source ~/.zshrc` or `source ~/.bashrc` once in the current shell. After that, `jsh use` retargets the link, so Java commands in that open shell use the new JDK immediately. New bash and zsh shells read the updated profile automatically. Other shells are not configured automatically.

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
