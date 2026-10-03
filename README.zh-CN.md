<h1 align="center">j-switch</h1>

<p align="center">发现本机 JDK，下载 Temurin，在终端切换 Java 版本。</p>

<p align="center">
  <a href="https://github.com/nanocm/j-switch/releases/latest"><strong>下载</strong></a> ·
  <a href="README.md">English</a> ·
  <a href="RELEASE_NOTES.md">更新说明</a>
</p>

---

## 快速开始

**安装。** 下载适用于 Windows x64、Linux x64 或 macOS x64/ARM64 的发布包。将 `jsh.exe`（或 `jsh`）放在当前用户可写的目录，并把该目录加入 `PATH`。

**选择 JDK。** 以下命令可在 PowerShell、bash 和 zsh 中运行：

```text
jsh list
jsh download 21
jsh use 21
jsh current
```

**首次切换。** 第一次运行 `jsh use` 后，需要完成一次终端配置：

| 平台 | 首次操作 |
| --- | --- |
| Windows | 重新打开终端，加载当前用户的 `JAVA_HOME` 和 `PATH`。 |
| macOS / Linux | 在当前终端运行 `source ~/.zshrc` 或 `source ~/.bashrc`。 |

之后，只要终端从 `jsh-current` 下的 `bin` 目录查找 Java，再运行 `jsh use` 就能在当前终端生效。如果 `PATH` 中其他 Java 排在前面，`jsh current` 会提示路径不一致。

## 命令

| 要做什么 | 命令 |
| --- | --- |
| 查找已登记及托管的 JDK | `jsh list` |
| 额外搜索常见系统位置 | `jsh list --scan` |
| 清理失效的登记项 | `jsh list --prune` |
| 搜索可下载的 Temurin 版本 | `jsh search [关键词]` |
| 下载并登记 JDK | `jsh download <主版本>` |
| 切换已安装的 JDK | `jsh use <版本或ID>` |
| 检查当前 Java | `jsh current` |

`jsh list` 会分别保留相同版本的多个安装。如果一个版本匹配多个 JDK，请使用列表中的 ID 运行 `jsh use`。

## 自定义扫描目录

普通的 `jsh list` 会检查已登记的 JDK、托管的 `jdks` 目录、`JAVA_HOME` 和 `scan_dirs` 中的目录。只有加上 `--scan` 才会扩大到常见系统位置。

在可执行文件旁的 `config.json` 中添加扫描目录：

```json
{
  "current_jdk": null,
  "jdks": {},
  "download_dir": "downloads",
  "scan_dirs": ["D:\\Java", "E:\\SDKs"]
}
```

请按系统修改路径。相对路径以可执行文件所在目录为基准。`download_dir` 存放校验过的压缩包；安装后的 JDK 始终位于旁边的 `jdks` 目录。目前只支持从 Eclipse Temurin 下载。

## 构建

需要 Rust 1.88 或更新版本：

```text
cargo test --locked
cargo build --release --locked
```

基于 [YiTouch/j-switch](https://github.com/YiTouch/j-switch) fork · [MIT 许可证](LICENSE)
