# skim-run

A command-line utility and library for launching, searching, and managing applications and system services with a fast, interactive fuzzy finder interface.

## Features
- Launch and search installed applications quickly
- Calculator mode for instant calculations
- Systemd service management (start/stop/restart services)
- Hyprland/Hyprctl integration for window management
- Extensible command-line interface with multiple modes
- Built with [skim](https://github.com/skim-rs/skim) for high-performance fuzzy finding

## Installation
Install from [crates.io](https://crates.io/crates/skim-run):

```sh
cargo install skim-run --features <apps|calc|hyprland|systemd|all> [--no-default-features]
```
By default, `apps` and `calc` are enabled.
For example, if you want only the `apps` and `hyprctl-clients` modes: `cargo install skim-run --features apps,hyprland --no-default-features`. If you want everything: `cargo install skim-run --features all`

## About
This repository also serves as a showcase for the capabilities of [skim](https://github.com/skim-rs/skim), demonstrating how to integrate fuzzy finding into Rust-based workflows and tools.

## Usage
Run with a mode as the first argument:

- **apps**: Launch/search installed applications
  ```sh
  skim-run apps
  ```
- **calc**: Calculator mode (optionally evaluate an expression)
  ```sh
  skim-run calc
  ```
  notes: 
    - an eval mode is available: `skim-run calc --eval "2 + 2 * 10"`
    - the `calc` mode uses [eva](https://github.com/oppiliappan/eva) under the hood. This means that the previous result *should* be available as `_`
- **hyprctl-clients**: Hyprland/Hyprctl window management
  ```sh
  skim-run hyprctl-clients
  ```
- **systemd-services**: Manage systemd services
  ```sh
  skim-run systemd-services
  ```

See all options and help:
```sh
skim-run --help
```

## License
MIT
