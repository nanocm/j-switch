# j-switch

Manage JDKs from your terminal with `jsh`: find existing installations, download Eclipse Temurin, and switch the active Java version.

**[Download the latest release](https://github.com/nanocm/j-switch/releases/latest)** · [简体中文](README.zh-CN.md) · [Release notes](RELEASE_NOTES.md)

Release binaries are available for Windows x64, Linux x64, and macOS x64/ARM64.

## Quick start

Extract `jsh.exe` (or `jsh`) into a directory your account can write to, then add that directory to `PATH`. The tool stores its configuration and managed JDKs beside the executable.

```sh
jsh list
jsh download 21
jsh use 21
jsh current
```

The first `jsh use` needs one shell setup step:

| Platform | One-time step |
| --- | --- |
| Windows | Reopen the terminal to load the new user `JAVA_HOME` and `PATH`. |
| macOS / Linux | Run `source ~/.zshrc` or `source ~/.bashrc` in the current shell. |

Later switches take effect in an open terminal when Java resolves from the `bin` directory under `jsh-current`. If another Java comes first on `PATH`, `jsh current` shows the mismatch.

## Commands

| Command | What it does |
| --- | --- |
| `jsh list` | Show registered JDKs and check managed installs, `JAVA_HOME`, and configured scan directories. |
| `jsh list --scan` | Also search common system locations; this can be slow. |
| `jsh list --prune` | Remove registrations whose JDK paths are unavailable. |
| `jsh search [keyword]` | Search available Temurin versions. |
| `jsh download <major>` | Download, verify, install, and register a Temurin JDK. |
| `jsh use <version-or-ID>` | Switch to an installed JDK. |
| `jsh current` | Show the active JDK and check the Java found on `PATH`. |

Installations sharing a version keep separate IDs. If `jsh use 21` matches more than one JDK, select one by its ID from `jsh list`.

## Configuration

`config.json` is stored beside `jsh`. To scan other locations, add `scan_dirs` there; ordinary `jsh list` does not search all system drives. For example:

```json
{
  "current_jdk": null,
  "jdks": {},
  "download_dir": "downloads",
  "scan_dirs": ["D:\\Java", "E:\\SDKs"]
}
```

Relative paths are resolved from the executable's directory. `download_dir` controls the archive cache; installed JDKs always go into the adjacent `jdks` directory. Downloads currently use Eclipse Temurin only; custom mirrors are not supported.

## Build from source

Requires Rust 1.88 or newer:

```sh
cargo test --locked
cargo build --release --locked
```

Forked from [YiTouch/j-switch](https://github.com/YiTouch/j-switch) · [MIT license](LICENSE)
