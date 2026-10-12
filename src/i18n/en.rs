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

Keys: p opens and closes Preferences, q or Ctrl+C quits.
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
    ("monitor_hint", "p: Preferences   q or Ctrl+C: quit"),
    // The Preferences page. The settings_* keys and their strings are
    // colony-ui's (colony_ui::i18n), copied exactly: every Colony program
    // says the same thing in the same words.
    ("settings_title", "Preferences"),
    ("settings_close", "Close"),
    ("settings_cat_general", "General"),
    (
        "settings_general_desc",
        "Preferences are saved automatically.",
    ),
    ("settings_cat_appearance", "Appearance"),
    (
        "settings_appearance_desc",
        "Adjust the theme, accents, text and visual effects.",
    ),
    ("settings_cat_accessibility", "Accessibility"),
    (
        "settings_accessibility_desc",
        "Make the interface easier to see, read and use.",
    ),
    ("settings_cat_about", "About"),
    // Collector's own.
    (
        "preferences_hint",
        "↑↓ Move   Tab Column   ←→ Change   Enter Open   Esc Close",
    ),
    ("preferences_section_language", "Language"),
    (
        "preferences_language_desc",
        "Applies at once, to every screen and message.",
    ),
    ("preferences_language", "Interface language"),
    ("preferences_language_system", "Same as the system"),
    ("language_en", "English"),
    ("language_fr", "Français"),
    ("preferences_section_refresh", "Refresh"),
    (
        "preferences_refresh_desc",
        "How often the readings are taken. More often costs more CPU.",
    ),
    ("preferences_refresh_interval", "Interval"),
    ("seconds", "{value} s"),
    (
        "preferences_about_desc",
        "Collector, the Colony terminal system monitor.",
    ),
    ("preferences_version", "Version {version}"),
    ("preferences_license", "License: {license}"),
    ("preferences_source", "Source code: {url}"),
    ("preferences_file", "Preferences file:"),
    (
        "preferences_set_aside",
        "Invalid preferences set aside as preferences.json.invalid",
    ),
    (
        "preferences_not_saved",
        "Preferences will not be saved: {error}",
    ),
    (
        "preferences_save_failed",
        "Could not save the preferences: {error}",
    ),
];
