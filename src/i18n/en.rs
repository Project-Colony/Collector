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
    // The Preferences page. The settings_* keys, theme_applied and their
    // strings are colony-ui's (colony_ui::i18n), copied exactly: every Colony
    // program says the same thing in the same words. A test holds both
    // locales to the `labels` of config/themes.json, which carries them.
    // Theme and accent names are not here: they come from that file too.
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
    ("settings_section_theme", "Theme"),
    (
        "settings_theme_desc",
        "Applies immediately to the whole interface.",
    ),
    ("settings_section_colors", "Colors & accents"),
    (
        "settings_colors_desc",
        "Pick an accent, or keep the one the theme was designed with.",
    ),
    ("settings_auto_accent", "Auto accent from background"),
    (
        "settings_auto_accent_desc",
        "Uses the theme's own accent instead of the one picked above.",
    ),
    ("settings_default_accent", "Theme accent"),
    ("settings_section_preview", "Preview"),
    (
        "settings_preview_desc",
        "A live sample of your current choices.",
    ),
    ("settings_preview_card", "Preview card"),
    ("settings_preview_body", "Body text looks like this."),
    ("settings_preview_button", "Button"),
    ("settings_section_vision", "Vision"),
    (
        "settings_vision_desc",
        "Options that make the interface easier to see.",
    ),
    ("settings_high_contrast", "High contrast"),
    (
        "settings_high_contrast_desc",
        "Increases the contrast of text and borders.",
    ),
    ("settings_section_motion", "Motion"),
    ("settings_motion_desc", "Calm the interface down."),
    ("settings_reduce_motion", "Reduce motion"),
    (
        "settings_reduce_motion_desc",
        "Turns off every animation and transition.",
    ),
    ("theme_applied", "Theme applied."),
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
    ("preferences_accent", "Accent"),
    ("preferences_on", "On"),
    ("preferences_off", "Off"),
    (
        "preferences_appearance_note",
        "Typography and Effects are not here: the terminal sets the font and its size, and Collector has no visual effects.",
    ),
    (
        "preferences_font_note",
        "The font is the terminal's: for a dyslexia-friendly one, such as OpenDyslexic Mono, pick it in the terminal's settings.",
    ),
    (
        "preferences_motion_note",
        "Nothing in Collector is animated: only the readings change, at the refresh interval. Anything animated added later will follow this setting.",
    ),
    (
        "preferences_reading_note",
        "Reading is not here: text size is the terminal's. Zoom with its settings or its keyboard shortcut.",
    ),
];
