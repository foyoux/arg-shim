# arg-spy

A simple command-line tool to spy on (capture) and copy command-line arguments directly to your clipboard.

## Features

- **Quick capture**: Joins all provided arguments with a single space and copies them to the system clipboard.
- **Cross-platform core**: Built with Rust and `arboard` for reliable clipboard access.
- **Windows Optimized**: Specifically supports Windows `x86_64` and `aarch64` (ARM64).

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

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
