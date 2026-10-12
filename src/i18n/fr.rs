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

Touches : q ou Ctrl+C pour quitter.
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
    ("quit_hint", "q ou Ctrl+C pour quitter"),
];
