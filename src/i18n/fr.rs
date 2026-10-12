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
    ("settings_section_theme", "Thème"),
    (
        "settings_theme_desc",
        "S'applique immédiatement à toute l'interface.",
    ),
    ("settings_section_colors", "Couleurs & accents"),
    (
        "settings_colors_desc",
        "Choisissez un accent, ou gardez celui du thème.",
    ),
    ("settings_auto_accent", "Accent automatique selon le fond"),
    (
        "settings_auto_accent_desc",
        "Utilise l'accent du thème plutôt que celui choisi ci-dessus.",
    ),
    ("settings_default_accent", "Accent du thème"),
    ("settings_section_preview", "Aperçu"),
    (
        "settings_preview_desc",
        "Un aperçu en direct de vos choix actuels.",
    ),
    ("settings_preview_card", "Carte d'aperçu"),
    (
        "settings_preview_body",
        "Le texte courant ressemble à ceci.",
    ),
    ("settings_preview_button", "Bouton"),
    ("settings_section_vision", "Vision"),
    (
        "settings_vision_desc",
        "Options pour rendre l'interface plus lisible.",
    ),
    ("settings_high_contrast", "Contraste élevé"),
    (
        "settings_high_contrast_desc",
        "Augmente le contraste du texte et des bordures.",
    ),
    ("settings_section_motion", "Mouvement"),
    ("settings_motion_desc", "Apaisez l'interface."),
    ("settings_reduce_motion", "Réduire les animations"),
    (
        "settings_reduce_motion_desc",
        "Désactive toutes les animations et transitions.",
    ),
    ("theme_applied", "Thème appliqué."),
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
    ("preferences_accent", "Accent"),
    ("preferences_on", "Activé"),
    ("preferences_off", "Désactivé"),
    (
        "preferences_appearance_note",
        "La typographie et les effets ne sont pas ici : le terminal choisit la police et sa taille, et Collector n'a pas d'effets visuels.",
    ),
    (
        "preferences_font_note",
        "La police est celle du terminal : pour une police adaptée à la dyslexie, comme OpenDyslexic Mono, choisissez-la dans ses réglages.",
    ),
    (
        "preferences_motion_note",
        "Rien n'est animé dans Collector : seuls les relevés changent, à l'intervalle d'actualisation. Toute animation ajoutée plus tard suivra ce réglage.",
    ),
    (
        "preferences_reading_note",
        "La lecture n'est pas ici : la taille du texte est celle du terminal. Zoomez dans ses réglages ou avec son raccourci clavier.",
    ),
];
