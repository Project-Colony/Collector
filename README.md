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
> filtered.

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
  connection and no file written (see the [privacy policy](#privacy-policy)).

Today btop and bottom show far more. The [roadmap](#roadmap) closes that gap,
and adds English and French and the Colony palettes on the way.

## What it does

One screen, refreshed every second:

- **CPU**: overall usage, as a gauge.
- **Memory**: used and total, as a gauge.
- **Disks**: used and total space, by mount point.
- **Network**: receive and transmit rates in KB/s, all interfaces except
  loopback together.
- **Processes**: the ten using the most CPU, with their share of the whole
  machine's CPU and their memory.

`q` or Ctrl+C quits, and the terminal is restored.

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
| `q`, Ctrl+C | Quit |

The terminal is restored on `q`, on Ctrl+C and when Collector panics. On Linux
and macOS it is also restored on a quit signal from outside (SIGTERM, SIGHUP,
SIGINT, SIGQUIT): Collector notices it within a second, puts the terminal back,
then ends the way that signal would have ended it.

## Roadmap

Each step is one release.

- **0.2.0**: release pipeline, signed builds, `--help` and `--version`, and a terminal restored on quit and on panic.
- **0.3.0**: core rewrite: Linux reads `/proc` and `/sys` directly; CPU, memory, disk, network and process boxes with correct rates; the terminal restored after crashes too; English and French.
- **0.4.0**: Preferences page, Colony palettes and accents, mouse support.
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

### Privacy policy

Collector makes no network connection of any kind. It reads the statistics it
shows from the operating system of the machine it runs on and draws them in
your terminal; nothing leaves that machine. There is no telemetry, no
analytics, no account and no update check (Colony handles updates). Collector
writes no files.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
