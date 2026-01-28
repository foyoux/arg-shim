# arg-shim

Windows 下灵活的 CLI 参数垫片与转换器。捕获进程参数，并利用可配置的模式和模板重写它们。

## 功能特性

- **拦截与转换**：捕获一个工具（如 `putty`）的参数，并将其重写为另一个工具（如 `ssh`）的格式。
- **模式匹配**：简单易用的 `{variable}` 模式语法（例如：`-ssh {user}@{host}`）。
- **正则支持**：支持完整的正则表达式，用于复杂的参数解析。
- **模板引擎**：使用命名变量、位置参数（`{{1}}`）、环境变量（`{{ENV:PATH}}`）和默认值（`{{port | 22}}`）重构命令。
- **剪贴板集成**：自动将转换后的结果复制到剪贴板。
- **多步复制**：支持一次性将多个内容依次复制到剪贴板（配合 Win+V 历史记录使用）。
- **跨架构**：原生支持 Windows x64 和 ARM64。

## 安装

### 从 Releases 下载

请访问 [Releases](https://github.com/foyoux/arg-shim/releases) 页面下载适合您平台的预编译二进制文件。

### 源码安装

确保您已安装 [Rust](https://rustup.rs/)：

```bash
cargo install --path .
```

## 使用方法

### 基础用法（侦探模式）

默认情况下，如果未找到配置，`arg-shim` 只是简单地将所有参数拼接并复制到剪贴板。

```bash
arg-shim Hello World
# 剪贴板内容: "Hello World"
```

### 垫片模式（参数转换）

1.  **初始化配置**：
    运行 `arg-shim --init` 在当前目录生成一个初始的 `arg-shim.toml`。

    ```bash
    arg-shim --init
    ```

2.  **配置规则**：
    编辑 `arg-shim.toml` 来定义如何解析和转换参数。

3.  **测试配置**：
    使用 `--check` 测试规则，结果将打印到控制台而不会覆盖剪贴板。
    您可以使用 `--app <name>` 模拟以不同的可执行文件名称运行。

    ```bash
    # 模拟程序名为 "putty"
    arg-shim --check --app putty -ssh user@host
    # 输出: ssh -p 22 user@host (打印到控制台)
    ```

4.  **部署**：
    将 `arg-shim.exe` 重命名为目标程序名（例如 `putty.exe`），并将其放置在原程序所在的位置。

## 配置

`arg-shim` 按照以下顺序查找配置文件（优先使用第一个匹配项）：

1.  **当前工作目录**（仅当可执行文件名以 `arg-shim` 开头时才会检查，用于调试）：
    *   `./arg-shim.toml`
2.  **可执行文件所在目录**（`.exe` 文件所在的文件夹，正式部署位置）：
    *   `./arg-shim.toml`

> 注意：旧版本中支持的 `putty.toml` 或 `%APPDATA%` 等路径已不再支持。所有配置统一汇总到 `arg-shim.toml` 中，通过 `app_name` 区分规则。

### 配置结构

```toml
# 如果没有匹配到规则，是否将原始参数复制到剪贴板？
fallback_raw = true

# 使用多模板复制时的默认间隔延迟（毫秒）(默认: 1000)
default_delay_ms = 1000

[[rules]]
# 可选：仅当可执行文件名为 "putty" 时应用（区分大小写）
app_name = "putty"

# 策略 A：简单模式
# 自动提取 {user}, {host}, {port}
# pattern = "-ssh {user}@{host} -P {port}"

# 支持多个模式（累积匹配）
# 1. 所有匹配成功的模式都会被执行。
# 2. 捕获的变量会合并，如果变量名相同，后定义的模式会覆盖前面的值。
patterns = [
    "-ssh {user}@{host} -P {port}", # 尝试捕获所有变量
    "-ssh {user}@{host}",           # 兼容不带端口的情况
    "--extra {extra}"               # 额外捕获其他参数
]

# 策略 B：正则表达式（高级）
# regex = '''^--target\s+(?P<host>[a-zA-Z0-9.-]+)(\s+--port\s+(?P<port>\d+))?'''

# 输出模板
# 单个模板（向后兼容）
# template = "ssh -p {{port | 22}} {{user}}@{{host}}"

# 多模板（多步剪贴板）
# 最后一项将是当前活动的剪贴板内容。
# 之前的项将进入剪贴板历史（Win+V）。
templates = [
    "{{password}}",       # 先复制
    "ssh {{user}}@{host}" # 后复制（当前激活）
]

# 可选：覆盖此规则的全局延迟
delay = 500 
```

### 模板语法

- **命名变量**：`{{user}}`（从 pattern 或 regex 捕获）
- **位置参数**：`{{1}}`（第1个参数），`{{2}}`（第2个参数）...
- **程序名称**：`{{0}}` 或 `{{EXE_NAME}}`
- **原始参数**：`{{RAW_ARGS}}`
- **当前目录**：`{{CWD}}`
- **环境变量**：`{{ENV:USERNAME}}`, `{{ENV:PATH}}`
- **默认值**：`{{port | 22}}`（如果 'port' 缺失或为空，则使用 '22'）