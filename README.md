# find-active

A simple, efficient Rust utility for **Hyprland** that implements "run or raise" functionality. It checks if a specific window class is currently active.
- If the window **exists**, it focuses it.
- If the window **does not exist**, it executes a specified launch command.

## Usage

```bash
find-active <CLASS> --exec <COMMAND>
```

### Arguments

- `<CLASS>`: The exact window class to search for (case-sensitive as per Hyprland). You can find this by running `hyprctl clients` in your terminal.
- `-e, --exec <COMMAND>`: The shell command to execute if the window is not found.

## Installation

### From the AUR

If you are using Arch Linux, you can install `find-active-git` from the AUR using an AUR helper like `paru` or `yay`:

```bash
paru -S find-active-git
```

### From Source

1. Clone the repository:
   ```bash
   git clone https://github.com/snewman-aa/find-active.git
   cd find-active
   ```

2. Build and install:
   ```bash
   cargo install --path .
   ```

## Examples

### Hyprland Configuration

This tool is designed to be used directly in your Hyprland keybindings. This allows you to have a single key that either switches to an app or launches it.

Add the following to your Hyprland config:

```hyprlang
# Open zen or focus it if it's already running
bind = SUPER, W, exec, find-active zen --exec zen-desktop

# Focus vesktop if it's running, otherwise launch it with wayland flags
bind = SUPER, D, exec, find-active vesktop -e 'vesktop --enable-features=UseOzonePlatform --ozone-platform=wayland'
```

### Troubleshooting

If the window isn't focusing, double-check the window class name using:

```bash
hyprctl clients
```

Look for the `class: ` field in the output for the application you are trying to target.
