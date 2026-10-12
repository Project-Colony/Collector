//! What the Preferences page sets, saved to
//! `<config>/preferences/preferences.json` the moment it changes: there is no
//! Save button.
//!
//! The file is read as a JSON value and taken field by field. A field that is
//! missing or of the wrong type falls back to its default and the rest of the
//! file is kept, so an old file, a hand-edited one or one written by a newer
//! build still loads. Only a file that cannot be read or is not JSON at all
//! fails to load, and the monitor then sets it aside rather than saving over it.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::i18n::{Locale, t, tf};
use crate::{paths, theme};

/// The theme a fresh install starts on, and the one an unknown theme falls
/// back to, as in colony-ui.
pub const DEFAULT_THEME: (&str, &str) = ("gruvbox", "dark");

/// Seconds between two readings, as the Preferences page offers them.
pub const REFRESH_INTERVALS: [f64; 4] = [0.5, 1.0, 2.0, 5.0];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Language {
    /// Whatever the system language is, read again at each start.
    #[default]
    System,
    En,
    Fr,
}

impl Language {
    pub const ALL: [Self; 3] = [Self::System, Self::En, Self::Fr];

    /// The value written to the file.
    fn key(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::En => "en",
            Self::Fr => "fr",
        }
    }

    pub fn locale(self) -> Locale {
        match self {
            Self::System => Locale::system(),
            Self::En => Locale::En,
            Self::Fr => Locale::Fr,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Preferences {
    // The fields of colony_ui::preferences::StandardPreferences that mean
    // something in a terminal, under its names and in its JSON forms, so the
    // file reads like every other Colony program's. The font, text size and
    // animations fields are left out: the terminal owns the font, and
    // Collector has no effects for Appearance to switch off. Reduce motion
    // stays, as Accessibility's Motion setting.
    pub selected_theme: String,
    pub selected_variant: String,
    /// An accent key such as `"violet"`, or `None` for the theme's own.
    /// Left out of the file when `None`, as colony-ui does.
    pub selected_accent: Option<String>,
    pub auto_accent: bool,
    pub high_contrast: bool,
    pub reduce_motion: bool,
    // Collector's own.
    pub language: Language,
    /// Seconds, one of [`REFRESH_INTERVALS`].
    pub refresh_interval: f64,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            selected_theme: DEFAULT_THEME.0.to_owned(),
            selected_variant: DEFAULT_THEME.1.to_owned(),
            selected_accent: None,
            auto_accent: false,
            high_contrast: false,
            reduce_motion: false,
            language: Language::System,
            refresh_interval: 1.0,
        }
    }
}

impl Preferences {
    /// Fails only when `text` is not JSON. Anything else that is wrong falls
    /// back to its default, one field at a time, and unknown fields are
    /// ignored; JSON that is not an object is all defaults.
    pub fn from_json(text: &str) -> serde_json::Result<Self> {
        let value: Value = serde_json::from_str(text)?;
        let string = |key| value.get(key).and_then(Value::as_str).map(str::to_owned);
        let flag = |key, default| value.get(key).and_then(Value::as_bool).unwrap_or(default);
        let defaults = Self::default();
        Ok(Self {
            selected_theme: string("selected_theme").unwrap_or(defaults.selected_theme),
            selected_variant: string("selected_variant").unwrap_or(defaults.selected_variant),
            selected_accent: string("selected_accent"),
            auto_accent: flag("auto_accent", defaults.auto_accent),
            high_contrast: flag("high_contrast", defaults.high_contrast),
            reduce_motion: flag("reduce_motion", defaults.reduce_motion),
            language: string("language")
                .and_then(|key| Language::ALL.into_iter().find(|l| l.key() == key))
                .unwrap_or(defaults.language),
            refresh_interval: value
                .get("refresh_interval")
                .and_then(Value::as_f64)
                .filter(|seconds| REFRESH_INTERVALS.contains(seconds))
                .unwrap_or(defaults.refresh_interval),
        })
    }

    /// Moves `setting` `by` places through its choices, wrapping around at
    /// either end. A toggle flips whichever way it is moved.
    pub fn step(&mut self, setting: Setting, by: isize) {
        match setting {
            Setting::Language => self.language = cycle(&Language::ALL, self.language, by),
            Setting::RefreshInterval => {
                self.refresh_interval = cycle(&REFRESH_INTERVALS, self.refresh_interval, by);
            }
            // Through the family's variants. Another family's row starts on
            // its first variant going forwards, its last going backwards.
            Setting::Theme(family) => {
                let Some(family_meta) = theme::catalog().families.get(family) else {
                    return;
                };
                let count = family_meta.variants.len() as isize;
                let current = family_meta.variants.iter().position(|variant| {
                    self.selected_theme == family_meta.key && self.selected_variant == variant.key
                });
                let variant = match current {
                    Some(at) => (at as isize + by).rem_euclid(count),
                    None if by < 0 => count - 1,
                    None => 0,
                };
                self.set_theme(family, variant as usize);
            }
            // The theme's own accent, then each shared one.
            Setting::Accent => {
                let accents = &theme::catalog().accents;
                let choices: Vec<Option<&str>> = std::iter::once(None)
                    .chain(accents.iter().map(|accent| Some(accent.key.as_str())))
                    .collect();
                let current = theme::picked_accent(self).map(|accent| accent.key.as_str());
                self.selected_accent = cycle(&choices, current, by).map(str::to_owned);
                self.auto_accent = false;
            }
            Setting::AutoAccent => self.auto_accent = !self.auto_accent,
            Setting::HighContrast => self.high_contrast = !self.high_contrast,
            Setting::ReduceMotion => self.reduce_motion = !self.reduce_motion,
        }
    }

    /// The theme picker's cards: `variant` of `family`, by position in
    /// themes.json. Out of range does nothing.
    pub fn set_theme(&mut self, family: usize, variant: usize) {
        let families = &theme::catalog().families;
        if let Some(family) = families.get(family)
            && let Some(variant) = family.variants.get(variant)
        {
            self.selected_theme.clone_from(&family.key);
            self.selected_variant.clone_from(&variant.key);
        }
    }

    /// An accent swatch, by position in themes.json. Picking one turns auto
    /// accent off, as in colony-ui.
    pub fn set_accent(&mut self, accent: usize) {
        if let Some(accent) = theme::catalog().accents.get(accent) {
            self.selected_accent = Some(accent.key.clone());
            self.auto_accent = false;
        }
    }

    /// Pretty-printed, for whoever opens the file by hand.
    pub fn to_json(&self) -> String {
        let mut value = json!({
            "selected_theme": self.selected_theme,
            "selected_variant": self.selected_variant,
            "auto_accent": self.auto_accent,
            "high_contrast": self.high_contrast,
            "reduce_motion": self.reduce_motion,
            "language": self.language.key(),
            "refresh_interval": self.refresh_interval,
        });
        if let Some(accent) = &self.selected_accent {
            value["selected_accent"] = accent.as_str().into();
        }
        format!("{value:#}\n")
    }
}

/// A setting the Preferences page changes with Left and Right.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setting {
    Language,
    RefreshInterval,
    /// A family of the theme picker, by position in themes.json.
    Theme(usize),
    Accent,
    AutoAccent,
    HighContrast,
    ReduceMotion,
}

/// The choice `by` places from `current`, wrapping around. A current value
/// that is not one of the choices counts as the first.
fn cycle<T: Copy + PartialEq>(choices: &[T], current: T, by: isize) -> T {
    let at = choices.iter().position(|&c| c == current).unwrap_or(0);
    let len = choices.len() as isize;
    choices[(at as isize + by).rem_euclid(len) as usize]
}

/// `<config>/preferences/preferences.json`, the design/filesystem.md tree.
pub fn path() -> io::Result<PathBuf> {
    Ok(paths::config_dir()?
        .join("preferences")
        .join("preferences.json"))
}

/// The preferences at `path`, or the defaults when there is no file yet. An
/// error means the file exists but cannot be read or is not JSON; it is left
/// untouched.
pub fn load(path: &Path) -> io::Result<Preferences> {
    match fs::read_to_string(path) {
        Ok(text) => Preferences::from_json(&text).map_err(io::Error::other),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Preferences::default()),
        Err(err) => Err(err),
    }
}

/// Writes through a temporary file in the same directory, then renames it
/// over the old one: a crash or a full disk mid-write leaves the previous file
/// whole rather than half of a new one.
pub fn save(path: &Path, preferences: &Preferences) -> io::Result<()> {
    let dir = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(dir)?;
    let temporary = with_suffix(path, ".tmp");
    let written = File::create(&temporary).and_then(|mut file| {
        file.write_all(preferences.to_json().as_bytes())?;
        file.sync_all()
    });
    let result = written.and_then(|()| fs::rename(&temporary, path));
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

/// Where the monitor saves, and what the status line tells the user about the
/// file, given what [`path`] and [`load`] returned. A file that failed to load
/// is renamed to `preferences.json.invalid` first, so that saving never
/// overwrites something the user may want back. If even that fails, nothing
/// is saved this session.
pub fn save_target(
    path: io::Result<PathBuf>,
    loaded: &io::Result<Preferences>,
) -> (Option<PathBuf>, Option<String>) {
    let not_saved = |err: io::Error| {
        let message = tf("preferences_not_saved", &[("error", &err.to_string())]);
        (None, Some(message))
    };
    match (path, loaded) {
        (Err(err), _) => not_saved(err),
        (Ok(path), Ok(_)) => (Some(path), None),
        (Ok(path), Err(_)) => match fs::rename(&path, with_suffix(&path, ".invalid")) {
            Ok(()) => (Some(path), Some(t("preferences_set_aside").to_owned())),
            Err(err) => not_saved(err),
        },
    }
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    name.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh directory under the system temp dir, removed on drop.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("collector-test-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            Self(dir)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn an_empty_object_is_the_defaults() {
        assert_eq!(
            Preferences::from_json("{}").unwrap(),
            Preferences::default()
        );
        assert_eq!(
            Preferences::from_json("[1, 2]").unwrap(),
            Preferences::default()
        );
    }

    #[test]
    fn a_bad_value_falls_back_and_keeps_the_rest() {
        let p = Preferences::from_json(
            r#"{
                "selected_theme": 42,
                "selected_variant": "latte",
                "selected_accent": ["violet"],
                "high_contrast": "yes",
                "reduce_motion": true,
                "language": "de",
                "refresh_interval": 0.25,
                "font_size": "large"
            }"#,
        )
        .unwrap();
        let defaults = Preferences::default();
        assert_eq!(p.selected_theme, defaults.selected_theme);
        assert_eq!(p.selected_accent, None);
        assert!(!p.high_contrast);
        assert_eq!(p.language, Language::System);
        assert_eq!(p.refresh_interval, 1.0);
        // The valid neighbours survive.
        assert_eq!(p.selected_variant, "latte");
        assert!(p.reduce_motion);
    }

    #[test]
    fn settings_cycle_through_their_choices_both_ways() {
        let mut p = Preferences::default();
        p.step(Setting::Language, 1);
        assert_eq!(p.language, Language::En);
        p.step(Setting::Language, -2);
        assert_eq!(p.language, Language::Fr, "wraps around backwards");
        p.step(Setting::Language, 1);
        assert_eq!(p.language, Language::System, "wraps around forwards");

        p.step(Setting::RefreshInterval, 1);
        assert_eq!(p.refresh_interval, 2.0);
        p.step(Setting::RefreshInterval, -3);
        assert_eq!(p.refresh_interval, 5.0);
    }

    #[test]
    fn theme_and_accent_rows_pick_from_themes_json() {
        let position = |key| {
            let families = &theme::catalog().families;
            families.iter().position(|f| f.key == key).unwrap()
        };
        let theme = |p: &Preferences| (p.selected_theme.clone(), p.selected_variant.clone());
        let mut p = Preferences::default();
        p.step(Setting::Theme(position("catppuccin")), 1);
        assert_eq!(theme(&p), ("catppuccin".into(), "latte".into()));
        p.step(Setting::Theme(position("catppuccin")), -1);
        assert_eq!(p.selected_variant, "mocha", "wraps around backwards");
        p.step(Setting::Theme(position("gruvbox")), -1);
        assert_eq!(
            theme(&p),
            ("gruvbox".into(), "dark".into()),
            "starts on the last"
        );
        p.set_theme(usize::MAX, 0);
        assert_eq!(theme(&p), ("gruvbox".into(), "dark".into()), "out of range");

        p.step(Setting::Accent, 1);
        assert_eq!(p.selected_accent.as_deref(), Some("red"));
        p.step(Setting::Accent, -2);
        assert_eq!(p.selected_accent.as_deref(), Some("amber"), "wraps around");
        p.auto_accent = true;
        p.step(Setting::Accent, 1);
        assert_eq!(
            p.selected_accent.as_deref(),
            Some("red"),
            "from the theme's own"
        );
        assert!(!p.auto_accent, "picking an accent turns auto accent off");
        p.step(Setting::Accent, -1);
        assert_eq!(p.selected_accent, None, "the theme's own again");
        p.set_accent(6);
        assert_eq!(p.selected_accent.as_deref(), Some("violet"));

        for (setting, flag) in [
            (
                Setting::AutoAccent,
                (|p: &Preferences| p.auto_accent) as fn(&Preferences) -> bool,
            ),
            (Setting::HighContrast, |p| p.high_contrast),
            (Setting::ReduceMotion, |p| p.reduce_motion),
        ] {
            p.step(setting, -1);
            assert!(flag(&p), "{setting:?} turns on");
            p.step(setting, 1);
            assert!(!flag(&p), "{setting:?} turns off");
        }
    }

    #[test]
    fn it_round_trips_through_the_file() {
        let dir = TempDir::new("round-trip");
        let path = dir.0.join("preferences").join("preferences.json");
        let preferences = Preferences {
            selected_theme: "catppuccin".into(),
            selected_variant: "mocha".into(),
            selected_accent: Some("violet".into()),
            auto_accent: true,
            high_contrast: true,
            reduce_motion: true,
            language: Language::Fr,
            refresh_interval: 0.5,
        };
        save(&path, &preferences).unwrap();
        assert_eq!(load(&path).unwrap(), preferences);
        assert!(!with_suffix(&path, ".tmp").exists(), "temporary file left");

        // No accent: left out, as colony-ui writes it, and read back as none.
        let preferences = Preferences::default();
        save(&path, &preferences).unwrap();
        assert!(
            !fs::read_to_string(&path)
                .unwrap()
                .contains("selected_accent")
        );
        assert_eq!(load(&path).unwrap(), preferences);
    }

    #[test]
    fn a_missing_file_is_the_defaults() {
        let dir = TempDir::new("missing");
        assert_eq!(
            load(&dir.0.join("preferences.json")).unwrap(),
            Preferences::default()
        );
    }

    #[test]
    fn an_invalid_file_is_set_aside_and_never_overwritten() {
        let dir = TempDir::new("invalid");
        fs::create_dir_all(&dir.0).unwrap();
        let path = dir.0.join("preferences.json");
        fs::write(&path, "{ not json").unwrap();

        let loaded = load(&path);
        assert!(loaded.is_err());
        let (target, status) = save_target(Ok(path.clone()), &loaded);
        assert_eq!(target.as_deref(), Some(path.as_path()));
        assert!(status.is_some(), "the user is told");
        assert_eq!(
            fs::read_to_string(with_suffix(&path, ".invalid")).unwrap(),
            "{ not json"
        );

        // The next save starts a new file next to the one set aside.
        save(&path, &Preferences::default()).unwrap();
        assert_eq!(load(&path).unwrap(), Preferences::default());
        assert!(with_suffix(&path, ".invalid").exists());
    }

    #[test]
    fn no_config_directory_means_nothing_is_saved() {
        let (target, status) = save_target(
            Err(io::Error::new(io::ErrorKind::NotFound, "no home")),
            &Ok(Preferences::default()),
        );
        assert_eq!(target, None);
        assert!(status.is_some());
    }

    #[test]
    fn the_file_sits_under_preferences_in_the_config_dir() {
        let path = path().unwrap();
        assert!(
            path.ends_with(
                Path::new("Colony")
                    .join("Collector")
                    .join("preferences")
                    .join("preferences.json")
            )
        );
    }
}
