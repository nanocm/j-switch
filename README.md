<h1 align="center">j-switch</h1>

<p align="center">Find JDKs on your machine. Download Temurin. Switch the Java used by your terminal.</p>

<p align="center">
  <a href="https://github.com/nanocm/j-switch/releases/latest"><strong>Download</strong></a> ·
  <a href="README.zh-CN.md">简体中文</a> ·
  <a href="RELEASE_NOTES.md">Release notes</a>
</p>

---

## Get started

**Install.** Download the release for Windows x64, Linux x64, or macOS x64/ARM64. Put `jsh.exe` (or `jsh`) in a directory your account can write to, and add that directory to `PATH`.

**Choose a JDK.** These commands work in PowerShell, bash, and zsh:

```text
jsh list
jsh download 21
jsh use 21
jsh current
```

**First switch.** Complete one setup step after the first `jsh use`:

| Platform | One-time step |
| --- | --- |
| Windows | Reopen the terminal to load your user `JAVA_HOME` and `PATH`. |
| macOS / Linux | Run `source ~/.zshrc` or `source ~/.bashrc` in the current shell. |

Later switches take effect in an open terminal when Java resolves from the `bin` directory under `jsh-current`. If another Java comes first on `PATH`, `jsh current` shows the mismatch.

## Commands

| Task | Command |
| --- | --- |
| Find registered and managed JDKs | `jsh list` |
| Search common system locations too | `jsh list --scan` |
| Remove unavailable registrations | `jsh list --prune` |
| Search downloadable Temurin versions | `jsh search [keyword]` |
| Download and register a JDK | `jsh download <major>` |
| Select an installed JDK | `jsh use <version-or-ID>` |
| Check the active Java | `jsh current` |

`jsh list` preserves installations sharing a version. If a version matches more than one JDK, use the displayed ID with `jsh use`.

## Scan your own directories

By default, `jsh list` checks registered JDKs, the managed `jdks` directory, `JAVA_HOME`, and directories in `scan_dirs`. It does not search entire drives unless you add `--scan`.

Edit `config.json` beside the executable to add directories:

```json
{
  "current_jdk": null,
  "jdks": {},
  "download_dir": "downloads",
  "scan_dirs": ["D:\\Java", "E:\\SDKs"]
}
```

Use paths for your system. Relative paths start from the executable's directory. `download_dir` stores verified archives; installed JDKs stay in the adjacent `jdks` directory. Downloads currently use Eclipse Temurin only.

## Build

With Rust 1.88 or newer:

```text
cargo test --locked
cargo build --release --locked
```

Forked from [YiTouch/j-switch](https://github.com/YiTouch/j-switch) · [MIT license](LICENSE)
