//! The interface in English or French. Every string the user reads goes
//! through [`t`] or [`tf`], and every number with a decimal part through
//! [`number`], so French gets its own words, units and decimal comma.
//!
//! The locale is process-wide, as in colony-ui: only the event loop draws.

mod en;
mod fr;

use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Locale {
    En,
    Fr,
}

impl Locale {
    /// French when the language subtag is `fr` (`fr`, `fr-FR`, `fr_CA.UTF-8`),
    /// English for anything else, `C` and `POSIX` included.
    pub fn from_tag(tag: &str) -> Self {
        let language = tag.split(['-', '_', '.', '@']).next().unwrap_or_default();
        if language.eq_ignore_ascii_case("fr") {
            Self::Fr
        } else {
            Self::En
        }
    }

    /// The system's language: on Linux the first of `LANGUAGE`, `LC_ALL`,
    /// `LC_MESSAGES` and `LANG` that is set, on Windows and macOS the user's
    /// preferred language.
    pub fn system() -> Self {
        sys_locale::get_locale().map_or(Self::En, |tag| Self::from_tag(&tag))
    }

    fn strings(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::En => en::STRINGS,
            Self::Fr => fr::STRINGS,
        }
    }
}

static FRENCH: AtomicBool = AtomicBool::new(false);

pub fn set_locale(locale: Locale) {
    FRENCH.store(locale == Locale::Fr, Ordering::Relaxed);
}

pub fn locale() -> Locale {
    if FRENCH.load(Ordering::Relaxed) {
        Locale::Fr
    } else {
        Locale::En
    }
}

/// The string for `key` in the current locale.
pub fn t(key: &'static str) -> &'static str {
    t_in(locale(), key)
}

/// Falls back to English, then to the key itself. The tests below keep both
/// fallbacks unreachable.
pub fn t_in(locale: Locale, key: &'static str) -> &'static str {
    lookup(locale, key)
        .or_else(|| lookup(Locale::En, key))
        .unwrap_or(key)
}

fn lookup(locale: Locale, key: &str) -> Option<&'static str> {
    locale
        .strings()
        .iter()
        .find(|(k, _)| *k == key)
        .map(|&(_, value)| value)
}

/// [`t`] with each `{name}` replaced by its value in `args`. One pass, so a
/// value that itself contains `{...}` (a process name, an argument) is left as
/// it is.
pub fn tf(key: &'static str, args: &[(&str, &str)]) -> String {
    let mut rest = t(key);
    let mut out = String::with_capacity(rest.len());
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let filled = after.find('}').and_then(|close| {
            let name = &after[..close];
            let (_, value) = args.iter().find(|(arg, _)| *arg == name)?;
            Some((close, *value))
        });
        match filled {
            Some((close, value)) => {
                out.push_str(value);
                rest = &after[close + 1..];
            }
            None => {
                out.push('{');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// `value` rounded to `decimals` places, with the locale's decimal separator:
/// `1.5` in English, `1,5` in French.
pub fn number(value: f64, decimals: usize) -> String {
    let text = format!("{value:.decimals$}");
    match locale() {
        Locale::En => text,
        Locale::Fr => text.replace('.', ","),
    }
}

/// Runs `f` in `locale`. The locale is process-wide and tests run in
/// parallel, so every test that depends on it goes through here.
#[cfg(test)]
pub fn with_locale<R>(locale: Locale, f: impl FnOnce() -> R) -> R {
    use std::sync::{Mutex, PoisonError};
    static LOCK: Mutex<()> = Mutex::new(());
    let _guard = LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    let previous = self::locale();
    set_locale(locale);
    let result = f();
    set_locale(previous);
    result
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    fn keys(strings: &[(&'static str, &str)]) -> BTreeSet<&'static str> {
        strings.iter().map(|&(key, _)| key).collect()
    }

    fn placeholders(text: &str) -> BTreeSet<&str> {
        text.split('{')
            .skip(1)
            .filter_map(|part| part.split_once('}').map(|(name, _)| name))
            .collect()
    }

    #[test]
    fn fr_and_en_have_identical_key_sets() {
        assert_eq!(keys(en::STRINGS), keys(fr::STRINGS));
    }

    /// One key, one string: a key listed twice would carry two values.
    #[test]
    fn no_key_is_listed_twice() {
        for strings in [en::STRINGS, fr::STRINGS] {
            assert_eq!(keys(strings).len(), strings.len());
        }
    }

    #[test]
    fn no_string_is_empty() {
        for (key, value) in en::STRINGS.iter().chain(fr::STRINGS) {
            assert!(!value.trim().is_empty(), "{key} is empty");
        }
    }

    #[test]
    fn both_locales_fill_the_same_placeholders() {
        for &(key, english) in en::STRINGS {
            assert_eq!(
                placeholders(english),
                placeholders(t_in(Locale::Fr, key)),
                "{key}"
            );
        }
    }

    /// A typo in a key would show the key itself on screen.
    #[test]
    fn every_key_used_in_the_code_exists() {
        let known = keys(en::STRINGS);
        let mut dirs = vec![PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src"))];
        let mut missing = Vec::new();
        while let Some(dir) = dirs.pop() {
            for entry in fs::read_dir(&dir).expect("read src") {
                let path = entry.expect("read src").path();
                if path.is_dir() {
                    dirs.push(path);
                    continue;
                }
                let source = fs::read_to_string(&path).expect("read a source file");
                for call in ["t(", "tf("] {
                    for (at, _) in source.match_indices(call) {
                        // `t(` and not `split(`, `print(` or the list above.
                        let before = source[..at].chars().next_back();
                        if before.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '"') {
                            continue;
                        }
                        // rustfmt may put the key on the next line.
                        let Some(rest) = source[at + call.len()..].trim_start().strip_prefix('"')
                        else {
                            continue;
                        };
                        let key = &rest[..rest.find('"').unwrap_or(0)];
                        if !known.contains(key) {
                            missing.push(format!("{}: {key}", path.display()));
                        }
                    }
                }
            }
        }
        assert!(missing.is_empty(), "keys missing from en.rs: {missing:?}");
    }

    /// The settings_* strings and theme_applied are colony-ui's. Resources
    /// ships them in config/themes.json under `labels`, and the themes.json
    /// CI job keeps that copy current, so a reworded label fails here.
    #[test]
    fn shared_strings_match_the_colony_labels() {
        let themes: serde_json::Value =
            serde_json::from_str(include_str!("../../config/themes.json")).expect("themes.json");
        for (locale, strings) in [("en", en::STRINGS), ("fr", fr::STRINGS)] {
            let shared = strings
                .iter()
                .filter(|(key, _)| key.starts_with("settings_") || *key == "theme_applied");
            for &(key, value) in shared {
                let label = themes["labels"][locale][key].as_str();
                assert_eq!(label, Some(value), "{locale}: {key}");
            }
        }
    }

    #[test]
    fn french_follows_the_language_subtag() {
        for tag in ["fr", "fr-FR", "fr_CA.UTF-8", "FR-be", "fr@euro"] {
            assert_eq!(Locale::from_tag(tag), Locale::Fr, "{tag}");
        }
        for tag in ["en-US", "C", "POSIX", "", "frr-DE", "de_DE.UTF-8"] {
            assert_eq!(Locale::from_tag(tag), Locale::En, "{tag}");
        }
    }

    #[test]
    fn numbers_use_the_locale_decimal_separator() {
        assert_eq!(with_locale(Locale::En, || number(1.26, 1)), "1.3");
        assert_eq!(
            with_locale(Locale::Fr, || number(1536.0 / 1024.0, 1)),
            "1,5"
        );
        assert_eq!(with_locale(Locale::Fr, || number(42.0, 0)), "42");
    }

    #[test]
    fn placeholders_are_filled_once() {
        let line = with_locale(Locale::Fr, || {
            tf("used_of_total_gb", &[("used", "{total}"), ("total", "2,0")])
        });
        assert_eq!(line, "{total} / 2,0 Go");
        let unknown = with_locale(Locale::En, || tf("percent", &[]));
        assert_eq!(unknown, "{value}%");
    }
}
