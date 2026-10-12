//! English, the canonical locale: each key is named after its English string,
//! and a new string is written here first. `fr.rs` must have the same keys.

pub(super) const STRINGS: &[(&str, &str)] = &[
    // Command line. `collector --version` is not here: it stays exactly
    // `collector X.Y.Z` in every locale (see `cli::print_version`).
    (
        "usage",
        "\
Usage: collector [OPTION]

A terminal system monitor: CPU, memory, disks, network and processes.

Options:
  -h, --help     Print this help and exit
  -V, --version  Print the version and exit

Keys: q or Ctrl+C quits.
",
    ),
    (
        "unexpected_argument",
        "collector: unexpected argument '{arg}'",
    ),
    (
        "needs_terminal",
        "Collector needs a terminal (with ssh, use ssh -t)",
    ),
    ("error", "collector: {error}"),
    ("signal_error", "cannot handle signal {signal}: {error}"),
    // The monitor.
    ("cpu", "CPU"),
    ("memory", "Memory"),
    ("disks", "Disks"),
    ("network", "Network"),
    ("top_processes", "Top Processes"),
    ("process", "Process"),
    ("rx_rate", "RX: {rate} KB/s"),
    ("tx_rate", "TX: {rate} KB/s"),
    ("used_of_total_gb", "{used} / {total} GB"),
    ("megabytes", "{value} MB"),
    ("percent", "{value}%"),
    ("quit_hint", "q or Ctrl+C to quit"),
];
