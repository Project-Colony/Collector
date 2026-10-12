//! French. Same keys as `en.rs`, written as French rather than translated word
//! for word: French units (Go, Mo, Ko), a space before `:` and `%`.

pub(super) const STRINGS: &[(&str, &str)] = &[
    (
        "usage",
        "\
Utilisation : collector [OPTION]

Un moniteur système pour le terminal : processeur, mémoire, disques, réseau et processus.

Options :
  -h, --help     Affiche cette aide et quitte
  -V, --version  Affiche la version et quitte

Touches : p ouvre et ferme les Préférences, q ou Ctrl+C quitte.
",
    ),
    (
        "unexpected_argument",
        "collector : argument inattendu « {arg} »",
    ),
    (
        "needs_terminal",
        "Collector a besoin d'un terminal (avec ssh, utilisez ssh -t)",
    ),
    ("error", "collector : {error}"),
    (
        "signal_error",
        "impossible d'intercepter le signal {signal} : {error}",
    ),
    ("cpu", "CPU"),
    ("memory", "Mémoire"),
    ("disks", "Disques"),
    ("network", "Réseau"),
    ("top_processes", "Processus les plus actifs"),
    ("process", "Processus"),
    ("rx_rate", "Reçu : {rate} Ko/s"),
    ("tx_rate", "Envoyé : {rate} Ko/s"),
    ("used_of_total_gb", "{used} / {total} Go"),
    ("megabytes", "{value} Mo"),
    ("percent", "{value} %"),
    ("monitor_hint", "p : Préférences   q ou Ctrl+C : quitter"),
    ("settings_title", "Préférences"),
    ("settings_close", "Fermer"),
    ("settings_cat_general", "Général"),
    (
        "settings_general_desc",
        "Les préférences sont enregistrées automatiquement.",
    ),
    ("settings_cat_appearance", "Apparence"),
    (
        "settings_appearance_desc",
        "Ajustez le thème, les accents, le texte et les effets visuels.",
    ),
    ("settings_cat_accessibility", "Accessibilité"),
    (
        "settings_accessibility_desc",
        "Rendez l'interface plus facile à voir, à lire et à utiliser.",
    ),
    ("settings_cat_about", "À propos"),
    (
        "preferences_hint",
        "↑↓ Déplacer   Tab Colonne   ←→ Modifier   Entrée Ouvrir   Échap Fermer",
    ),
    ("preferences_section_language", "Langue"),
    (
        "preferences_language_desc",
        "S'applique tout de suite, à tous les écrans et messages.",
    ),
    ("preferences_language", "Langue de l'interface"),
    ("preferences_language_system", "Comme le système"),
    ("language_en", "English"),
    ("language_fr", "Français"),
    ("preferences_section_refresh", "Actualisation"),
    (
        "preferences_refresh_desc",
        "La fréquence des relevés. Plus souvent coûte plus de processeur.",
    ),
    ("preferences_refresh_interval", "Intervalle"),
    ("seconds", "{value} s"),
    (
        "preferences_about_desc",
        "Collector, le moniteur système de Colony pour le terminal.",
    ),
    ("preferences_version", "Version {version}"),
    ("preferences_license", "Licence : {license}"),
    ("preferences_source", "Code source : {url}"),
    ("preferences_file", "Fichier des préférences :"),
    (
        "preferences_set_aside",
        "Préférences invalides mises de côté : preferences.json.invalid",
    ),
    (
        "preferences_not_saved",
        "Les préférences ne seront pas enregistrées : {error}",
    ),
    (
        "preferences_save_failed",
        "Impossible d'enregistrer les préférences : {error}",
    ),
];
