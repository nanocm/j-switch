# j-switch

用 `jsh` 管理 JDK：查找已有安装、下载 Eclipse Temurin，并在终端切换 Java 版本。

**[下载最新版本](https://github.com/nanocm/j-switch/releases/latest)** · [English](README.md) · [更新说明](RELEASE_NOTES.md)

提供 Windows x64、Linux x64、macOS x64/ARM64 的发布包。

## 快速开始

将 `jsh.exe`（或 `jsh`）解压到当前用户可写的目录，并把该目录加入 `PATH`。配置文件和托管的 JDK 会保存在可执行文件旁。

```sh
jsh list
jsh download 21
jsh use 21
jsh current
```

首次运行 `jsh use` 后，需要完成一次终端配置：

| 平台 | 首次操作 |
| --- | --- |
| Windows | 重新打开终端，加载当前用户的 `JAVA_HOME` 和 `PATH`。 |
| macOS / Linux | 在当前终端运行 `source ~/.zshrc` 或 `source ~/.bashrc`。 |

之后，只要终端从 `jsh-current` 下的 `bin` 目录查找 Java，再运行 `jsh use` 就能在当前终端生效。如果 `PATH` 中其他 Java 排在前面，`jsh current` 会提示路径不一致。

## 命令

| 命令 | 作用 |
| --- | --- |
| `jsh list` | 列出已登记的 JDK，并检查托管目录、`JAVA_HOME` 和配置的扫描目录。 |
| `jsh list --scan` | 额外搜索常见系统位置，可能较慢。 |
| `jsh list --prune` | 清理路径已失效的登记项。 |
| `jsh search [关键词]` | 搜索可下载的 Temurin 版本。 |
| `jsh download <主版本>` | 下载、校验、安装并登记 Temurin JDK。 |
| `jsh use <版本或ID>` | 切换到已安装的 JDK。 |
| `jsh current` | 显示当前 JDK，并检查 `PATH` 中的 Java。 |

相同版本的多个安装会保留不同 ID。如果 `jsh use 21` 匹配多个 JDK，请使用 `jsh list` 显示的 ID。

## 配置

`config.json` 位于 `jsh` 旁。可在其中添加 `scan_dirs`；普通的 `jsh list` 不会扫描所有系统磁盘。例如：

```json
{
  "current_jdk": null,
  "jdks": {},
  "download_dir": "downloads",
  "scan_dirs": ["D:\\Java", "E:\\SDKs"]
}
```

相对路径以可执行文件所在目录为基准。`download_dir` 设置压缩包缓存；安装后的 JDK 始终放在旁边的 `jdks` 目录。目前只支持从 Eclipse Temurin 下载，不支持自定义镜像。

## 从源码构建

需要 Rust 1.88 或更新版本：

```sh
cargo test --locked
cargo build --release --locked
```

基于 [YiTouch/j-switch](https://github.com/YiTouch/j-switch) fork · [MIT 许可证](LICENSE)
