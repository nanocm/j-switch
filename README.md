# jsh - JDK Switch Helper

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows-blue.svg)](https://github.com/YiTouch/j-switch)

[English](#english-documentation) | [中文文档](README_CN.md)

## English Documentation

A fast, cross-platform JDK version management and switching command-line tool written in Rust.

### ✨ Features

- 🔍 **JDK Discovery**: `list` checks registered JDKs, managed downloads, configured `scan_dirs`, and `JAVA_HOME`; `list --scan` searches system locations
- 🔄 **Quick Switching**: Switch between different JDK versions with a single command
- ⚙️ **Environment Management**: Automatically updates `JAVA_HOME` and `PATH` environment variables
- 📦 **Version Management**: List, install, and manage multiple JDK versions
- 💾 **Persistent Configuration**: Saves JDK configuration in `config.json`
- 🎨 **Beautiful Interface**: Colorful and intuitive command-line interface

### 📋 Table of Contents

- [System Requirements](#-system-requirements)
- [Important Notes](#️-important-notes)
- [Installation](#-installation)
- [Quick Start](#-quick-start)
- [Command Reference](#-command-reference)
- [Usage Examples](#-usage-examples)
- [Configuration File](#️-configuration-file)
- [Troubleshooting](#-troubleshooting)
- [Contributing](#-contributing)
- [License](#-license)
- [Acknowledgments](#-acknowledgments)
- [Roadmap](#️-roadmap)

### 🔧 System Requirements

- **Operating System**: Windows 10+
- **Rust**: 1.70 or higher (only required for compilation)
- **JDK**: At least one JDK installation (version 7+)

### ⚠️ Important Notes

> **JDK Path Scanning Depth**  
> `jsh list --scan` searches common system locations (drive roots C: through G: on Windows) up to 5 levels deep and may take time. Ordinary `jsh list` scans only managed downloads and directories named in `scan_dirs`.

> **JDK Download Source**  
> This tool uses the following official sources for JDK downloads:
> - **Primary Source**: [Adoptium (Eclipse Temurin)](https://adoptium.net/) - Provides high-quality, TCK-certified OpenJDK binaries
> - **API Interface**: Retrieves available version lists and download links through Adoptium's official API
> - **Resource Hosting**: Download resources are hosted on GitHub for stability and reliability
>
> All downloaded JDKs are officially certified OpenJDK distributions, ensuring security and reliability.

### 📦 Installation

#### Method 1: Download Pre-compiled Binary

Download the latest version from the [Releases](https://github.com/YiTouch/j-switch/releases) page.

#### Method 2: Build from Source

```bash
# Clone the repository
git clone https://github.com/YiTouch/j-switch.git
cd j-switch

# Build release version
cargo build --release

# Binary located at target/release/jsh.exe (Windows)
```

### 🚀 Quick Start

1. **List all detected JDKs**:
```bash
jsh list
```

Run `jsh list --scan` once to discover JDKs installed elsewhere. Found installations are registered for later fast listings.

2. **Switch to a specific JDK version**:
```bash
jsh use 17
```

3. **View currently active JDK**:
```bash
jsh current
```

### 📝 Command Reference

| Command | Description | Example |
|---------|-------------|---------|
| `jsh list` | List registered JDKs, managed downloads, `scan_dirs`, and `JAVA_HOME` | `jsh list` |
| `jsh list --scan` | Search common system locations and register found JDKs | `jsh list --scan` |
| `jsh current` | Display currently active JDK | `jsh current` |
| `jsh use <version or ID>` | Switch JDKs; use the ID from `list` when a version has multiple installations | `jsh use 17` |
| `jsh download <version>` | Download, install, and register a JDK | `jsh download 21` |
| `jsh search [version]` | Search downloadable JDK versions | `jsh search 17` |
| `jsh --help` | Display help information | `jsh --help` |

### 💡 Usage Examples

Downloaded JDKs live in the `jdks` directory next to `jsh.exe` and are registered immediately. `jsh list` shows a stable ID for each installation. `jsh use 17` works when only one JDK 17 is registered; if there are several, use `jsh use <ID>` with the ID shown by `list`. A full version such as `17.0.10` also works when it has one match. Existing major-version keys in `config.json` are migrated automatically.

#### List JDK Installations

```bash
$ jsh list

Scanning for JDK installations...

Installed JDKs:
================================================================================
* JDK 17 (current)
  ID: 17-17.0.10-0123456789abcdef
  Use: jsh use 17
  Version: 17.0.10
  Vendor: OpenJDK
  Path: C:\Program Files\Java\jdk-17

- JDK 11
  Version: 11.0.8
  Path: C:\Program Files\Java\jdk-11.0.8

- JDK 8
  Version: 1.8.0_291
  Vendor: Oracle
  Path: C:\Program Files\Java\jdk1.8.0_291

--------------------------------------------------------------------------------
Total: 3 JDKs
```

#### Switch JDK Version

On Windows, run `jsh use <version or ID>` as Administrator once. jsh creates a `jsh-current` junction next to the executable and points the system `JAVA_HOME` and `PATH` at that stable location. Reopen the terminal once. Later `jsh use` commands only retarget the junction, so `java` changes immediately in the open terminal without elevation.

```bash
$ jsh use 11

Selected JDK:
  Version: JDK 11
  Path: C:\Program Files\Java\jdk-11.0.8

Activating JDK...
[OK] System JAVA_HOME now points to: C:\Tools\jsh\jsh-current
[OK] System PATH now starts with: C:\Tools\jsh\jsh-current\bin
[OK] Active JDK junction: C:\Tools\jsh\jsh-current -> C:\Program Files\Java\jdk-11.0.8

[OK] Successfully switched to JDK

One-time setup:
  Reopen this terminal after the stable JDK path is added to JAVA_HOME and PATH.
```

#### View Current JDK

```bash
$ jsh current

Current JDK:
============================================================
Version: JDK 11
Full Version: 11.0.8
Path: C:\Program Files\Java\jdk-11.0.8
============================================================

[OK] JAVA_HOME is correctly set
```

### ⚙️ Configuration File

jsh stores configuration in `config.json` next to the executable. Add directories to `scan_dirs` to search them on every `jsh list`; each directory is scanned up to 5 levels deep. A JDK root itself is also accepted. Relative paths are resolved from the executable's directory.

Configuration example:
```json
{
  "jdks": {
    "11-11.0.8-fedcba9876543210": {
      "path": "C:\\Program Files\\Java\\jdk-11.0.8",
      "version": "11",
      "vendor": null,
      "java_version": "11.0.8"
    },
    "17-17.0.10-0123456789abcdef": {
      "path": "C:\\Program Files\\Java\\jdk-17",
      "version": "17",
      "vendor": "OpenJDK",
      "java_version": "17.0.10"
    }
  },
  "current_jdk": "17-17.0.10-0123456789abcdef",
  "download_dir": "C:\\path\\to\\jsh\\downloads",
  "scan_dirs": ["D:\\Java", "E:\\SDKs"]
}
```

#### Adding to PATH

**Windows**:
```powershell
# Temporarily add to PATH (current session)
$env:Path += ";D:\xx\xx\(jsh.exe)"

# Permanently add to PATH (run PowerShell as Administrator)
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";D:\xx\xx\(jsh.exe)", "User")
```

### 🐛 Troubleshooting

#### JDK Not Detected

If your JDK is not automatically detected:

1. **Configure directories or run a full scan**: Add paths to `scan_dirs` in `config.json`, or run `jsh list --scan`. The latter searches drive roots C: through G: on Windows, up to 5 levels deep.

#### Environment Variables Not Updated (Windows)

1. **Reopen the terminal once after initial setup**; later switches affect the open terminal immediately.
2. **Run initial setup as Administrator** to update system `JAVA_HOME` and `PATH`; later switches only retarget the junction.
3. **Verify** with `java -version` and `$env:JAVA_HOME` in PowerShell.

### 🤝 Contributing

Contributions are welcome!

### 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

### 🙏 Acknowledgments

- Inspired by the nvm tool

### 🗺️ Roadmap

- [x] Automatic JDK detection
- [x] JDK version switching
- [x] Environment variable management
- [x] JDK download and installation
- [x] Version search functionality

---

Made with ❤️ and 🦀 Rust
