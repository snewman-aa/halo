# Halo & Hypraise

A *run or raise* utility and radial menu designed specifically for Hyprland.

This project is split into two components:
- **Hypraise**: The core logic library and a standalone CLI tool. It handles window management (via Hyprland's IPC), desktop entry parsing, and fuzzy matching.
- **Halo**: A GTK4-based radial menu that depends on Hypraise for its core functionality. Built with [*Realm4*](https://github.com/Realm4/realm4)

Those who want the *run or raise* functionality for their own keybinds without a GUI can install and use **Hypraise** on its own.

## Motivation

I love a keyboard-centered experience when I'm writing code, texts, emails, etc. Activities that use the keyboard. But if I'm just chilling on
discord, watching videos, browsing the internet, basically *consuming* rather than *creating*, I find myself just holding my mouse.

In these cases, relying on my keybinds, something that typically speeds me up when I'm *creating*, slows me down when I'm *consuming*.
I wanted a shell that allows me to quickly navigate my most used apps without needing to transition from browse-mode to write-mode.

**Halo** is my attempt to create that.

## Features

- **Run or Raise:** If an app is running, it focuses it; otherwise, it launches it
- **XDG Utilization:** Parses `.desktop` entries to resolve icons, window classes, and execution strings
- **Radial Menu (Halo):** A quick-access menu that appears at your cursor for mouse-driven navigation
- **Dynamic Theming:** Extracts colors from your active GTK theme
- **Live Configuration:** Updates slots and mappings automatically when `config.toml` changes

## Installation

### AUR (Arch Linux)

The easiest way to install on Arch Linux is via the AUR:

```bash
# To install everything (GUI + CLI)
paru -S halo-git

# To install only the CLI logic (if you don't want the GUI)
paru -S hypraise-git
```

### Build from Source

You will need **Rust** installed. If building the GUI (**Halo**), you also need **GTK4** and **gtk4-layer-shell**.

#### Option 1: Install everything
```bash
git clone https://github.com/snewman-aa/halo
cd halo
cargo install --path crates/hypraise
cargo install --path crates/halo
```

#### Option 2: Install only the CLI (Hypraise)
If you only want the *run or raise* logic for your Hyprland keybinds:
```bash
cargo install --path crates/hypraise
```

## Usage

### CLI (Hypraise)
`hypraise` can be used as a standalone utility for specific apps. It uses `.desktop` entries by default, or you can specify a target app class and launch command.

```bash
# Focus or launch Zen Browser
hypraise zen

# Use a specific class and command (skips desktop entry lookup)
hypraise "My App" --class "my-app-class" --exec "/path/to/app"
```

An example Hyprland keybind of mine:
```hyprlang
bind = Super, A, exec, hypraise zen
```

### GUI (Halo)
Halo runs as a daemon and provides the radial menu.

#### 1. Start Halo Daemon
Add it to your Hyprland config:
```hyprlang
exec-once = halo
```

#### 2. Trigger the Menu
To trigger the menu, use the `hypraise show` and (optionally) `hypraise hide` commands.

Using `bind` for show and `bindr` (release) for hide enables *hold-to-show* behavior:
```hyprlang
bind = SUPER, grave, exec, hypraise show
bindr = SUPER, grave, exec, hypraise hide
```

> [!NOTE]
> I haven't been able to get the hold-to-show behavior to work with mouse bindings (like Mouse 5).
> I use Mouse 5 `mouse:276` in my config to make it solely a mouse experience:
> `bind = ,mouse:276, exec, hypraise show`

### Halo Interaction
- **Flick** cursor toward an icon to *run-or-raise* it
- **Right Click** an icon to close the application (uses `killactive`)
- **Left Click** in the center or outside the icons to dismiss the menu

## Configuration

The config file is located at `~/.config/halo/config.toml`

If the file does not exist, Halo will present a *Setup* slot when first opened. Selecting this slot will generate a default configuration for you.

### Example `config.toml`

```toml
[[slots]]
direction = "North"
app = "zen" # browser

[[slots]]
direction = "East"
app = "ghostty" # terminal

[[slots]]
direction = "South"
app = "dolphin" # files

[[slots]]
direction = "West"
app = "spotify" # music

[[slots]]
direction = "SE"
app = "vesktop" # discord
```

### Slot Options

- `direction`: One of `North`, `NorthEast`, `East`, `SouthEast`, `South`, `SouthWest`, `West`, `NorthWest` (or short forms like `n`, `ne`, `0`, `1`)
- `app`: The name of the application (searches desktop entries)
- `class`: (Optional) The window class to match
- `exec`: (Optional) The command to execute

> [!NOTE]
> If both `class` and `exec` are provided, they will override the desktop entry.
