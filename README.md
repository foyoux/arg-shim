# arg-shim

A flexible CLI argument shim and transformer for Windows. Capture process arguments and rewrite them using configurable patterns and templates.

## Features

- **Intercept & Transform**: Capture arguments from one tool (e.g., `putty`) and rewrite them for another (e.g., `ssh`).
- **Pattern Matching**: Easy-to-use `{variable}` pattern syntax (e.g., `-ssh {user}@{host}`).
- **Regex Support**: Full Regular Expression support for complex argument parsing.
- **Template Engine**: Reconstruct commands using named variables, positional arguments (`{{1}}`), and default values (`{{port | 22}}`).
- **Clipboard Integration**: Automatically copies the transformed result to your clipboard.
- **Cross-Architecture**: Native support for Windows x64 and ARM64.

## Installation

### From Releases

Download the pre-compiled binaries for your platform from the [Releases](https://github.com/foyoux/arg-shim/releases) page.

### From Source

Ensure you have [Rust](https://rustup.rs/) installed:

```bash
cargo install --path .
```

## Usage

### Basic Usage (Spy Mode)

By default, if no configuration is found, `arg-shim` simply joins all arguments and copies them to the clipboard.

```bash
arg-shim Hello World
# Clipboard: "Hello World"
```

### Shim Mode (Transformation)

1.  **Initialize Configuration**:
    Run `arg-shim --init` to generate a starter `arg-shim.toml` in the current directory.

    ```bash
    arg-shim --init
    ```

2.  **Configure Rules**:
    Edit `arg-shim.toml` to define how arguments should be parsed and transformed.

3.  **Test Configuration**:
    Use `--check` to test your rules without overwriting the clipboard.

    ```bash
    arg-shim --check -ssh user@host
    # Output: ssh -p 22 user@host (printed to console)
    ```

4.  **Deploy**:
    Rename `arg-shim.exe` to the target program name (e.g., `putty.exe`) and place it where the original program was expected.

## Configuration

`arg-shim` looks for configuration files in the following order (first match wins):

1.  **Environment Variable**: `ARG_SHIM_CONFIG`
2.  **App-Specific Config**: `./<exe_name>.arg-shim.toml` (e.g., `putty.arg-shim.toml`)
3.  **App-Specific Generic**: `./<exe_name>.toml` (e.g., `putty.toml`)
4.  **Directory Generic**: `./arg-shim.toml`
5.  **Global User Config**: `%APPDATA%\arg-shim\config.toml`

### Configuration Structure

```toml
# If no rules match, should we copy the raw arguments to the clipboard?
fallback_raw = true

[[rules]]
# Optional: Only apply if the executable is named "putty" (case-insensitive)
app_name = "putty"

# Strategy A: Simple Pattern
# Automatically extracts {user}, {host}, {port}
pattern = "-ssh {user}@{host} -P {port}"

# Strategy B: Regex (Advanced)
# regex = '''...'''

# Output Template
template = "ssh -p {{port | 22}} {{user}}@{{host}}"
```

### Template Syntax

- **Named Variables**: `{{user}}` (captured from pattern/regex)
- **Positional Arguments**: `{{1}}`, `{{2}}` (1-based index of original arguments)
- **Program Name**: `{{0}}` or `{{EXE_NAME}}`
- **Raw Arguments**: `{{RAW_ARGS}}`
- **Default Values**: `{{port | 22}}` (Use '22' if 'port' is missing or empty)

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.