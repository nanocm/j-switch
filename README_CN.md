# j-switch (`jsh`)

适用于 Windows、macOS 和 Linux 的 JDK 管理工具。本仓库基于 [YiTouch/j-switch](https://github.com/YiTouch/j-switch) fork。

`jsh` 可以发现本机 JDK、下载 Eclipse Temurin JDK、分别保存相同版本的安装，并切换当前使用的 JDK。

## 安装

可从 [Releases](https://github.com/nanocm/j-switch/releases) 下载二进制文件，或使用 Rust 1.85 及以上版本从源码编译（项目采用 Rust 2024 edition）：

```sh
cargo build --release --locked
```

Windows 产物是 `target/release/jsh.exe`，其他系统是 `target/release/jsh`。请将可执行文件及其旁边的 `config.json`、`jdks`、`downloads` 和 Windows 上的 `jsh-current` 保存在同一位置。Windows 上请选一个普通用户可写的专用目录，不要把 `jsh.exe` 放在 `Program Files` 或 Windows 系统目录下。将可执行文件所在目录加入 `PATH` 后，即可在任意终端运行 `jsh`。

## 命令

| 命令 | 作用 |
| --- | --- |
| `jsh list` | 刷新托管目录、`JAVA_HOME` 和配置的扫描目录，列出已登记的 JDK。 |
| `jsh list --scan` | 额外搜索常见系统位置。Windows 上会扫描 C: 至 G: 盘、最多 5 层，可能较慢。 |
| `jsh list --prune` | 清理路径已失效的登记项。断开连接的磁盘上的 JDK 日后仍可重新扫描登记。 |
| `jsh current` | 显示 `JAVA_HOME` 指向的 JDK；未设置时使用已保存的选择。如果 `PATH` 中的 `java` 来自其他位置，会给出提醒。 |
| `jsh use <版本或ID>` | 切换到已登记的 JDK。 |
| `jsh search [关键词]` | 搜索可下载的 Eclipse Temurin 版本。 |
| `jsh download <主版本>` | 下载、校验、安装并登记该主版本最新的 Temurin JDK。 |

示例：

```sh
jsh list
jsh download 21
jsh list
jsh use 21
jsh current
```

`jsh list` 会显示每个安装的稳定 ID。某个主版本只有一个安装时，可以直接使用 `jsh use 21`；若有多个安装，请使用列表中的 ID。完整版本号在只匹配一个安装时也可使用。旧版配置中以主版本号为键的记录会自动迁移。

下载的文件会按 Adoptium 提供的大小和 SHA-256 校验。校验通过的压缩包留在下载缓存中，安装后的 JDK 位于可执行文件旁的 `jdks` 目录，并会立即出现在 `jsh list` 中。`--vendor temurin` 和 `--vendor adoptium` 是同一下载源的两个名称；目前不支持其他发行商或自定义镜像。

## 在当前终端切换

### Windows

首次运行 `jsh use <版本或ID>` 时，需要使用管理员终端设置**系统级** `JAVA_HOME` 和 `PATH`。`jsh` 会在可执行文件旁建立 `jsh-current` 目录联接，并把 `jsh-current\bin` 放在系统 `PATH` 的最前面。首次配置后重新打开一次终端，让终端继承这些环境变量。

以后运行 `jsh use` 只需切换目录联接目标。只要当前终端的 `JAVA_HOME` 指向 `jsh-current`，且 `java` 首先从 `jsh-current\bin` 找到，新版本会立即生效。若其他 Java 路径排在前面，`jsh current` 会提醒你。后续切换要求普通用户能够写入 `jsh.exe` 所在目录。

这项首次配置会改变其他终端和用户看到的系统 Java 版本。请将安装目录保存在自己可控制的位置，不要向其他用户开放写权限。

### macOS 和 Linux

`jsh use` 会更新 `.zshrc` 或 `.bashrc` 中由 jsh 管理的配置。若要在当前终端生效，请按所用 shell 运行 `source ~/.zshrc` 或 `source ~/.bashrc`。新终端会自动读取更新后的配置。

## 配置文件

`config.json` 位于可执行文件旁。`scan_dirs` 和 `download_dir` 中的相对路径以该目录为基准。扫描目录可以直接是 JDK 根目录；向下扫描最多 5 层。`jsh list` 会单独显示失效登记项，使用 `jsh list --prune` 可清理。

```json
{
  "current_jdk": null,
  "jdks": {},
  "download_dir": "downloads",
  "scan_dirs": ["D:\\Java", "E:\\SDKs"]
}
```

请按实际位置修改路径。`download_dir` 存放经过校验的压缩包；安装后的 JDK 始终位于 `jsh` 旁的 `jdks` 目录。移动工具时，使用相对 `download_dir` 更方便。

## 开发与验证

```sh
cargo test --locked
cargo build --release --locked
```

参阅 [v0.2.0 发布说明](RELEASE_NOTES.md)。本项目使用 [MIT 许可证](LICENSE)。
