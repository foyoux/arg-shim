# arg-spy

A simple command-line tool to spy on (capture) and copy command-line arguments directly to your clipboard.
Now evolving into a powerful **Argument Shim & Adapter**.

## Features

- **Quick capture**: Joins all provided arguments with a single space and copies them to the system clipboard.
- **Cross-platform core**: Built with Rust and `arboard` for reliable clipboard access.
- **Windows Optimized**: Specifically supports Windows `x86_64` and `aarch64` (ARM64).
- **Configurable Transformation** (Coming Soon): Transform captured arguments using patterns, regex, and templates into any format you need (e.g., Putty args -> OpenSSH command).

## Installation

### From Source

Ensure you have [Rust](https://rustup.rs/) installed:

```bash
cargo install --path .
```

### From Releases

Download the pre-compiled binaries for your platform from the [Releases](https://github.com/foyoux/arg-spy/releases) page.

## Usage

Simply pass any text or arguments to the command:

```bash
arg-spy Hello World!
# Clipboard now contains: "Hello World!"
```

Useful for capturing paths, complex flags, or any output you want to quickly move to another application.

---

## Design Whitepaper (v0.2 Planning)

We are actively developing a configuration-driven transformation engine. This will allow `arg-spy` to act as a shim for various tools.

### 1. Configuration Loading Priority

To support multiple "identities" (e.g., masquerading as `putty.exe` or `winscp.exe`), configuration is loaded in the following order (first match wins):

1.  **Environment Variable**: `ARG_SPY_CONFIG` (Highest priority, for debugging/scripts)
2.  **App-Specific Config**: `./<exe_name>.arg-spy.toml` (Recommended, e.g., `putty.arg-spy.toml`)
3.  **App-Specific Generic**: `./<exe_name>.toml` (e.g., `putty.toml` - Use with caution to avoid conflicts)
4.  **Directory Generic**: `./arg-spy.toml` (Shared rules for all tools in the folder)
5.  **Global User Config**: `%APPDATA%/arg-spy/config.toml` or `~/.config/arg-spy/config.toml`
6.  **Built-in Default**: Fallback to raw argument copying.

### 2. Configuration Structure

```toml
# Global setting: If no rules match, fallback to copying raw arguments? (Default: true)
fallback_raw = true

[[rules]]
# Optional: Only apply if the executable is named "putty" (or "putty.exe")
app_name = "putty"

# Strategy A: Simple Pattern Matching (like CLI help)
# Automatically extracts {user}, {host}, {port}
pattern = "-ssh {user}@{host} -P {port}"

# Strategy B: Regex (Advanced)
# regex = '''...'''

# Output Template
# Supports:
# - Named variables: {{user}}
# - Positional arguments: {{1}}, {{2}} (Original args index)
# - Default values: {{port | 22}}
template = "ssh -p {{port | 22}} {{user}}@{{host}}"
```

### 3. Variable System

*   **Named Variables**: Extracted via `pattern` (e.g., `{host}`) or `regex` named groups.
*   **Positional Variables**: `{{1}}`, `{{2}}` correspond to the 1st, 2nd argument from the original command. `{{0}}` is the program name.
*   **Built-ins**: `{{RAW_ARGS}}`, `{{CWD}}`.
*   **Pipes**: `{{var | default_value}}` provided fallback if variable is missing or empty.

## Roadmap / Todos

- [ ] **Core Logic**: Refactor `main.rs` to support a "Pipeline" architecture (Load Config -> Match -> Extract -> Render -> Output).
- [ ] **Config Loader**: Implement the 5-layer configuration loading strategy.
- [ ] **Parser Engine**:
    - [ ] Implement Regex extraction.
    - [ ] Implement Simple Pattern matching (convert `{var}` pattern to Regex).
- [ ] **Template Engine**:
    - [ ] Implement `{{var}}` replacement.
    - [ ] Implement `{{N}}` positional argument replacement.
    - [ ] Implement `{{var | default}}` syntax.
- [ ] **CLI Improvements**:
    - [ ] Add `--init` flag to generate a default configuration file.
    - [ ] Add `--check` or `--dry-run` to test configuration against arguments without modifying clipboard.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.