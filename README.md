# Halo & Hypraise

A radial menu and *run or raise* utility designed specifically for Hyprland, built with the [**Relm4**](https://github.com/Relm4/relm4) framework.

Halo provides a GTK4-based radial menu that appears at your cursor, allowing you to quickly switch between applications or launch new ones using directional flicks.

Hypraise is the companion CLI that handles the *run or raise* logic and communicates with the Halo daemon.

## Motivation

I love a keyboard-centered experience when I'm writing code, texts, emails, etc. Activities that use the keyboard. But if I'm just chilling on
discord, watching videos, browsing the internet, basically *consuming* rather than *creating*, I find myself just holding my mouse.

In these cases, relying on my keybinds, something that typically speeds me up when I'm *creating*, slows me down when I'm *consuming*.
I wanted a shell that allows me to quickly navigate my most used apps without needing to transition from browse-mode to write-mode.

**Halo** is my attempt to create that.

## Features

- **Radial Menu (Halo):** Your favorite apps, immediately surrounding your cursor at a moment's notice
- **Run or Raise:** Intelligent application switching. If an app is running, it focuses it, otherwise, it launches it
- **XDG Utilization:** Uses `.desktop` entries to find icons, window classes, and execution strings from app names
- **Dynamic Theming:** Automatically extracts colors from your active GTK theme
- **Live Configuration:** Updates slots and mappings when the configuration changes

## Installation

### AUR (Arch Linux)

The easiest way to install Halo on Arch Linux is via the AUR:

```bash
paru -S halo-git
```

## Usage

### 1. Start the Halo Daemon
Halo runs as a daemon. Add it to your execs in your Hyprland config:

```bash
# hyprland.conf
exec-once = halo
```

### 2. Configure Keybindings
To trigger the radial **Halo** menu, bind `hypraise show` and `hypraise hide` to a key. Using `bind` for show and `bindr` (release) for hide enables a sort of *hold-to-show* experience.

```hyprlang
# hyprland.conf
bind = SUPER, grave, exec, hypraise show
bindr = SUPER, grave, exec, hypraise hide
```

> [!NOTE]
> I haven't been able to get the hold-to-show behavior to work with mouse bindings (like Mouse 5)

I use Mouse 5 `mouse:276` in my config to make it a complete mouse experience. Holding a key like `Ctrl` allows apps like your browser to still capture Mouse 5 for *Forward*.

With this config, left click in the deadzone to close **Halo**:

```hyprlang
bind = ,mouse:276, exec, hypraise show
```

### 3. CLI "Run or Raise"
You can also use `hypraise` as a standalone utility for specific apps:

```bash
# Focus or launch Firefox
hypraise firefox

# Use a specific class and command
hypraise "My App" --class "my-app-class" --exec "/path/to/app"
```

I have a couple binds using this, a la

```hyprlang
bind = Super, A, exec, hypraise zen
```

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
> If both `class` and `exec` are provided, they will override the desktop entry

## TODOs

- [x] **Close via Right-Click:** Allow right-clicking an icon of an open application to close it (via Hyprland `closewindow`)
- [ ] **Live Application Updates:** File watcher for desktop entry directories to automatically refresh the app cache when new software is installed
- [ ] **Active Apps List:** List all currently running apps (with desktop entries) that aren't assigned to a radial slot (potentially with assigned keybinds)
- [ ] **Eye Candy:** Add animations for menu transitions and icon selection (low priority)
