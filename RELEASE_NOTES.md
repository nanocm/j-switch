# j-switch v0.3.0

Choose where downloaded JDKs are installed with `install_dir` in `config.json` beside `jsh`:

```json
{
  "install_dir": "managed-jdks",
  "download_dir": "downloads",
  "scan_dirs": []
}
```

`install_dir` holds extracted JDKs; `download_dir` remains the verified archive cache. Paths may be absolute or relative to the executable. Existing installations are not moved, and `jsh list` continues to discover the original `jdks` directory after you change `install_dir`.

The README now has a visual quick start and matching English and Chinese guides.

After the first `jsh use`, reopen the terminal once on Windows or source your shell profile once on macOS or Linux. Later switches can take effect in the open terminal.
