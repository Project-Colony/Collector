<div align="center">

# Collector

**A terminal system monitor for Project Colony: CPU, memory, disks, network and processes, live, in any terminal.**

</div>

[![License: GPL-3.0-or-later](https://img.shields.io/badge/License-GPL--3.0--or--later-blue.svg)](LICENSE)
[![Colony app](https://img.shields.io/badge/Colony-system-purple)](https://github.com/Project-Colony/Colony)
[![Platforms](https://img.shields.io/badge/platforms-linux%20%7C%20windows%20%7C%20macOS-lightgrey)](#installation)

Watching what a machine is doing usually means htop or btop in a terminal on
Linux and macOS, or Task Manager and Activity Monitor on the desktop: a
different tool on each system, each installed and updated its own way, outside
Colony. Collector is the Colony one: a terminal system monitor, installed and
updated through Colony, that runs the same on Linux, Windows and macOS,
locally, over SSH or inside tmux.

> **Status:** today's minimal monitor is used by hand on Linux and passes its
> tests, including a pseudo-terminal run, on Windows and macOS in CI. 0.2.0
> is the first release, and Colony installs it only with a valid signature.
> A btop-class rewrite is in progress (see the [roadmap](#roadmap) below).
> Known gap until then: the process list cannot be scrolled, sorted or
> filtered. Colours are 24-bit when the terminal says it shows them
> (`COLORTERM=truecolor` or `24bit`, or Windows Terminal), and the nearest of
> the xterm 256 colours otherwise.

## Why Collector

- **Installed and updated by Colony**, like the rest of the ecosystem: no
  package manager or download page per system. Every release asset is signed
  with the Project-Colony ed25519 release key, and Colony checks that
  signature before installing.
- **The same program, screen and keys on Linux, Windows and macOS.** Task
  Manager and Activity Monitor each belong to one system, and htop does not
  run on Windows. bottom runs on all three too; Collector is the one Colony
  installs, updates and checks.
- **In the terminal you already have open**: locally, over SSH (`ssh -t`) or
  inside tmux, where a desktop task manager cannot follow.
- **Small and quiet**: a single binary of about 1 MB on Linux, with no network
  connection and no file written except its own preferences (see
  [Privacy](#privacy)).

Today btop and bottom show far more. The [roadmap](#roadmap) closes that gap.

## What it does

One screen, refreshed every second, or at the interval chosen in
[Preferences](#preferences) (half a second to five seconds):

- **CPU**: overall usage, as a gauge.
- **Memory**: used and total, as a gauge.
- **Disks**: used and total space, by mount point.
- **Network**: receive and transmit rates in KB/s, all interfaces except
  loopback together.
- **Processes**: the ten using the most CPU, with their share of the whole
  machine's CPU and their memory.

`q` or Ctrl+C quits, and the terminal is restored. Clicking **Collector** in
the top-left corner, or pressing `p`, opens the [Preferences](#preferences).
Everything is shown in English or French (see [Language](#language)), in the
Colony theme and accent chosen there, with high contrast if asked for.

## Installation

Collector runs in a terminal. Started without one (output redirected to a
file, or `ssh host collector` without `-t`), it says so and exits.

### Via Colony (recommended)

Search for **Collector** in [Colony](https://github.com/Project-Colony/Colony)
and install it. Updates arrive through the launcher. Until Colony can open
terminal programs in a terminal window, start the installed binary from a
terminal.

### Direct binary download

Grab the asset for your platform from the
[latest release](../../releases/latest):

| Platform | Asset |
|---|---|
| Linux | `collector-linux` |
| Windows | `collector-windows.exe` |
| macOS (Apple Silicon) | `collector-macos` |
| macOS (Intel) | `collector-macos-x86` |

```bash
chmod +x collector-linux && ./collector-linux
```

Over SSH, ask for a terminal: `ssh -t host collector`.

### Build from source

```bash
git clone https://github.com/Project-Colony/Collector
cd Collector
cargo build --release --locked
```

Requires Rust 1.95 or newer. For development, `cargo run --profile fast` gives
release-level speed with fast rebuilds.

### Platforms

The same features on all three. How each one is tested:

| Platform | How it is tested |
|---|---|
| Linux | by hand before each release, and in CI |
| macOS | built and CI-tested only (unit tests and a pseudo-terminal smoke test); nobody has run it on a Mac yet |
| Windows | in CI: unit tests and a pseudo-terminal smoke test under ConPTY |

## Usage

```text
collector            start the monitor
collector --help     print usage and exit (-h)
collector --version  print the version and exit (-V)
```

| Key | Action |
|---|---|
| `p` | Open or close Preferences (or click **Collector**, top left) |
| `q`, Ctrl+C | Quit |

On the Preferences page:

| Key | Action |
|---|---|
| Up, Down | Move between categories, or between sections and settings |
| Tab, Shift+Tab | Switch between the categories and their content |
| Enter | Open or close a section, or move a setting to its next value |
| Left, Right | Change the focused setting; on a theme, move through its variants |
| Esc, `p` | Close Preferences (or click **Close**) |

The mouse wheel moves through the open category, and a click on a theme card
or an accent swatch picks it. Collector takes the mouse for this, so selecting
text with the mouse needs the terminal's own modifier, Shift in most
terminals.

The terminal is restored on `q`, on Ctrl+C and when Collector panics. On Linux
and macOS it is also restored on a quit signal from outside (SIGTERM, SIGHUP,
SIGINT, SIGQUIT): Collector notices it within a second, puts the terminal back,
then ends the way that signal would have ended it.

### Preferences

Click **Collector** in the top-left corner, or press `p`: the Preferences page
takes the place of the boxes until you close it. Its categories are General,
Appearance, Accessibility and About, and each section opens with Enter or a
click. A change applies at once and is saved at once; there is no Save button.

- **General**: the interface language, and how often the readings are taken
  (every 0.5, 1, 2 or 5 seconds; more often costs more CPU).
- **Appearance**: the theme, from every Colony theme family and its light and
  dark variants; the accent, one of the eight shared Colony accents or the
  theme's own; and a live preview. The themes come from
  [Project-Colony-Resources](https://github.com/Project-Colony/Project-Colony-Resources),
  the same ones every Colony program offers. Typography and Effects are left
  out: the terminal sets the font and its size, and Collector has no visual
  effects.
- **Accessibility**: high contrast, which moves text and borders away from the
  background of whichever theme is on, and reduce motion (Collector animates
  nothing today). A dyslexia-friendly font or larger text is set in the
  terminal.
- **About**: the version, the license, the source code and where the
  preferences file is.

The preferences are kept in `preferences/preferences.json` under Collector's
config directory:

| Platform | Preferences file |
|---|---|
| Linux | `~/.config/Colony/Collector/preferences/preferences.json` (or under `$XDG_CONFIG_HOME`) |
| Windows | `%LOCALAPPDATA%\Colony\Collector\preferences\preferences.json` |
| macOS | `~/Library/Application Support/Colony/Collector/preferences/preferences.json` |

A value Collector does not understand falls back to its default and the rest
of the file is kept. A file that is not valid JSON is renamed to
`preferences.json.invalid`, never overwritten, and Collector says so at the
bottom of the screen and starts from the defaults.

### Language

Collector speaks French when the system language is French, and English
otherwise, with French units and decimal commas (`1,5 Go`). On Linux the
language comes from the first of `LANGUAGE`, `LC_ALL`, `LC_MESSAGES` and `LANG`
that is set, so `LANGUAGE=en collector` runs it in English; on Windows and
macOS it is the language chosen in the system settings. The language chosen in
[Preferences](#preferences) overrides it, for `--help` and the error messages
too. `collector --version` prints the same `collector X.Y.Z` line in both.

## Roadmap

Each step is one release.

- **0.2.0**: release pipeline, signed builds, `--help` and `--version`, and a terminal restored on quit and on panic.
- **0.3.0**: English and French, a Preferences page, and the Colony themes and accents with high contrast.
- **0.4.0**: core rewrite: Linux reads `/proc` and `/sys` directly; CPU, memory, disk, network and process boxes with correct rates; the terminal restored after crashes too; mouse support.
- **0.5.0**: process depth: tree view, detail panel, signals, renice, filtering.
- **0.6.0**: sensors and hardware: temperatures, frequencies, disk I/O, battery.
- **0.7.0**: GPU monitoring.
- **1.0.0**: layouts, presets and terminal modes; btop feature parity on Linux.
- **1.1.0**: deeper macOS and Windows support.

## Documentation

How a release is cut, signed and published:
[docs/releases.md](docs/releases.md).

## Code signing policy

Free code signing provided by [SignPath.io](https://signpath.io), certificate by [SignPath Foundation](https://signpath.org).

Windows builds are Authenticode-signed this way once the SignPath Foundation
has accepted the project; until then they ship without Authenticode. Every
release asset, on every platform, is always signed with the Project-Colony
ed25519 release key, and Colony verifies that signature before installing.

Team roles and members:

- Committers and reviewers: [MotherSphere](https://github.com/MotherSphere)
- Approvers: [MotherSphere](https://github.com/MotherSphere)

## Privacy

Collector sends no telemetry, no analytics and no crash reports.

| Data | Stored or sent | Where, and why |
|---|---|---|
| Settings | stored | `Colony/Collector/preferences/preferences.json` in this machine's config directory: `~/.config` on Linux, `%LOCALAPPDATA%` on Windows, `~/Library/Application Support` on macOS (see [Preferences](#preferences)) |
| System statistics | neither | read from the operating system to draw the screen, kept in memory only |

Collector connects to no server at all: it has no account, no update check
(Colony handles updates) and no network code. Nothing it reads leaves the
machine it runs on.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
