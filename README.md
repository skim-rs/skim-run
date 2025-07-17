# skim-run

A fast, interactive toolbox for your Linux desktop—launch apps, calculate, manage windows, and control systemd services, all from your terminal. Powered by [skim](https://github.com/skim-rs/skim) for blazing fuzzy search.

## Features

- 🚀 **App Launcher**: Instantly fuzzy-search and launch installed applications.
- 🧮 **Calculator**: Evaluate expressions, with previous results available as `_`.
- 🛠️ **Systemd Manager**: Start, stop, restart, and inspect systemd services.
- 🪟 **Hyprland Window Tools**:
  - **hyprctl-clients**: Fuzzy-switch between open windows.
  - **hyprctl-hide**: Hide windows to a special workspace, swap or unhide them interactively.
- ⚡ **Extensible CLI**: Add your own modes, combine features, and script workflows.
- 🦾 **Built on skim**: Lightning-fast fuzzy finding for everything.


## Installation

Install from [crates.io](https://crates.io/crates/skim-run):

```sh
cargo install skim-run --features <apps|calc|hyprland|systemd|paru> [--no-default-features] [--all-features]
```

- By default, `apps` and `calc` are enabled.
- To install only specific modes (e.g. apps + hyprctl-clients):  
  ```sh
  cargo install skim-run --features apps,hyprland --no-default-features
  ```
- To enable everything:  
  ```sh
  cargo install skim-run --all-features
  ```


## Why skim-run?

- **One fuzzy interface for everything**: No more memorizing commands or hunting for windows.
- **Supercharge your workflow**: Launch, switch, hide, and manage—all with fuzzy search.
- **Showcase for [skim](https://github.com/skim-rs/skim)**: See how fuzzy finding can power real desktop tools.


## Usage

Each mode is a power tool—run with the mode name as the first argument:

### App Launcher
```sh
skim-run apps
```
Fuzzy-search and launch any installed application.

### Calculator
```sh
skim-run calc
```
- Evaluate interactively, or:
- One-shot eval:  
  ```sh
  skim-run calc --eval "2 + 2 * 10"
  ```
- Previous result available as `_` (e.g. `sqrt(_)`).

### Hyprland Window Management

#### Switch between open windows
```sh
skim-run hyprctl-clients
```

#### Hide, swap, and unhide windows (special workspace)
```sh
skim-run hyprctl-hide
```
- **Enter**: Swap the current window with a hidden one.
- **Alt-Enter**: Unhide a window (move it to your current workspace).
- Use this to keep your workspace clean and recall hidden windows instantly.

### Systemd Service Manager
```sh
skim-run systemd-services
```
- Start, stop, restart, and inspect services with fuzzy search.

### Paru/AUR Package Search (if enabled)
```sh
skim-run paru
```
- Fuzzy-search and manage AUR packages.

---

See all options and help:
```sh
skim-run --help
```

Combine modes with `--modes` for tab-switching between tools!


## License

MIT

