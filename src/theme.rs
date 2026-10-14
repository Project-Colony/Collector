//! The Colony themes and accents, read from `config/themes.json`: a
//! byte-for-byte copy of Project-Colony-Resources' `generated/themes.json`,
//! never edited here (CI fails when it falls behind Resources).
//!
//! colony-ui reads the same tokens, but it is built on iced, which a terminal
//! program cannot carry (see src/paths.rs). So the parts a terminal needs are
//! ported from its theme.rs: the (family, variant) resolver with its Gruvbox
//! dark fallback, the high-contrast derivation and `contrast_on` with the
//! ink and paper of `contrastEnds`.
//!
//! Every colour Collector draws comes from here: no other file names one.

use std::sync::{OnceLock, PoisonError, RwLock};

use ratatui::style::Color;
use serde_json::Value;

use crate::i18n::{self, Locale};
use crate::preferences::{DEFAULT_THEME, Preferences};

const THEMES_JSON: &str = include_str!("../config/themes.json");

/// A name in English and French, as themes.json carries it.
#[derive(Debug)]
pub struct Label {
    en: String,
    fr: String,
}

impl Label {
    /// The name in the current locale.
    pub fn text(&self) -> &str {
        match i18n::locale() {
            Locale::En => &self.en,
            Locale::Fr => &self.fr,
        }
    }

    fn from_json(label: &Value) -> Option<Self> {
        let text = |locale| label.get(locale)?.as_str().map(str::to_owned);
        Some(Self {
            en: text("en")?,
            fr: text("fr")?,
        })
    }
}

macro_rules! palette {
    ($($field:ident),* $(,)?) => {
        /// The palette fields Collector draws with, under their themes.json
        /// names. The rest of a theme has nothing to colour in a terminal.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct Palette {
            $(pub $field: Color,)*
            /// The accent in effect: the one picked in Preferences, or the
            /// theme's own (`accent_blue`).
            pub accent: Color,
            /// Legible text on `accent`.
            pub on_accent: Color,
        }

        impl Palette {
            fn from_json(palette: &Value, ends: ContrastEnds) -> Option<Self> {
                let color = |field| hex(palette.get(field)?);
                let accent = color("accent_blue")?;
                Some(Self {
                    $($field: color(stringify!($field))?,)*
                    accent,
                    on_accent: readable_end(accent, ends),
                })
            }

            fn map(self, f: impl Fn(Color) -> Color) -> Self {
                Self {
                    $($field: f(self.$field),)*
                    accent: f(self.accent),
                    on_accent: f(self.on_accent),
                }
            }
        }
    };
}

palette!(
    bg_primary,
    bg_card_hover,
    bg_selected,
    bg_progress,
    text_primary,
    text_secondary,
    text_muted,
    text_dim,
    success,
    success_bg,
    warning,
    border_subtle,
    divider,
);

impl Palette {
    /// colony-ui's `ThemePalette::with_high_contrast`, same amounts: text and
    /// borders move away from the background, which itself stays.
    fn with_high_contrast(mut self) -> Self {
        let light = is_light(self.bg_primary);
        // Darkened on a light background, brightened on a dark one.
        let push = |color, on_light: f32, on_dark: f32| {
            if light {
                shift(color, -on_light)
            } else {
                shift(color, on_dark)
            }
        };
        self.text_primary = push(self.text_primary, 0.15, 0.12);
        self.text_secondary = push(self.text_secondary, 0.12, 0.10);
        self.text_muted = push(self.text_muted, 0.10, 0.10);
        self.text_dim = push(self.text_dim, 0.10, 0.08);
        self.border_subtle = push(self.border_subtle, 0.15, 0.12);
        self.divider = push(self.divider, 0.15, 0.12);
        self
    }

    fn with_accent(self, accent: Color) -> Self {
        Self {
            accent,
            on_accent: contrast_on(accent),
            ..self
        }
    }
}

pub struct Variant {
    pub key: String,
    pub label: Label,
    /// The picker card: the variant's background, crossed by its accent.
    pub swatch_bg: Color,
    pub swatch_accent: Color,
    palette: Palette,
}

pub struct Family {
    pub key: String,
    pub label: Label,
    pub variants: Vec<Variant>,
}

/// One of the shared accents a user can pick over the theme's own.
pub struct Accent {
    pub key: String,
    pub label: Label,
    pub color: Color,
}

pub struct Catalog {
    /// In the order themes.json lists them, which is the pickers' order.
    pub families: Vec<Family>,
    pub accents: Vec<Accent>,
    /// Text on a fill: `contrastEnds`' ink, then paper.
    contrast_ends: ContrastEnds,
}

type ContrastEnds = [Color; 2];

/// Every family, variant and accent in themes.json, read once.
pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| parse(THEMES_JSON))
}

/// An entry with a missing or malformed field is left out rather than
/// failing: the tests hold the shipped file to its own counts.
fn parse(text: &str) -> Catalog {
    let json: Value = serde_json::from_str(text).unwrap_or_default();
    let list = |key| {
        json.get(key)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
    };
    let key = |value: &Value| value.get("key")?.as_str().map(str::to_owned);
    let label = |value: &Value| Label::from_json(value.get("label")?);

    // Pure black and white stand in for an end the file lacks.
    let ends = json.get("contrastEnds");
    let end = |name: &str, fallback| ends.and_then(|e| hex(e.get(name)?)).unwrap_or(fallback);
    let contrast_ends = [
        end("ink", Color::Rgb(0, 0, 0)),
        end("paper", Color::Rgb(255, 255, 255)),
    ];

    let variant = |value: &Value| {
        let swatch = value.get("swatch")?;
        Some(Variant {
            key: key(value)?,
            label: label(value)?,
            swatch_bg: hex(swatch.get("bg")?)?,
            swatch_accent: hex(swatch.get("accent")?)?,
            palette: Palette::from_json(value.get("palette")?, contrast_ends)?,
        })
    };
    let families = list("families")
        .filter_map(|value| {
            Some(Family {
                key: key(value)?,
                label: label(value)?,
                variants: value
                    .get("variants")?
                    .as_array()?
                    .iter()
                    .filter_map(variant)
                    .collect(),
            })
        })
        .collect();
    let accents = list("accents")
        .filter_map(|value| {
            Some(Accent {
                key: key(value)?,
                label: label(value)?,
                color: hex(value.get("color")?)?,
            })
        })
        .collect();
    Catalog {
        families,
        accents,
        contrast_ends,
    }
}

/// `"#rrggbb"`, the only form themes.json writes.
fn hex(value: &Value) -> Option<Color> {
    let digits = value.as_str()?.strip_prefix('#')?;
    if digits.len() != 6 || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let rgb = u32::from_str_radix(digits, 16).ok()?;
    let [_, r, g, b] = rgb.to_be_bytes();
    Some(Color::Rgb(r, g, b))
}

/// The palette of `family`/`variant`, or Gruvbox dark when either is unknown:
/// an old file, a hand-edited one or a removed family degrades instead of
/// failing.
pub fn resolve(family: &str, variant: &str) -> Palette {
    let find = |family: &str, variant: &str| {
        catalog()
            .families
            .iter()
            .find(|f| f.key == family)?
            .variants
            .iter()
            .find(|v| v.key == variant)
            .map(|v| v.palette)
    };
    find(family, variant)
        .or_else(|| find(DEFAULT_THEME.0, DEFAULT_THEME.1))
        .expect("themes.json has Gruvbox dark")
}

/// The accent the user picked, or `None` for the theme's own: while auto
/// accent is on, when none was picked, or when the key is not a shipped
/// accent.
pub fn picked_accent(prefs: &Preferences) -> Option<&'static Accent> {
    if prefs.auto_accent {
        return None;
    }
    let key = prefs.selected_accent.as_deref()?;
    catalog().accents.iter().find(|accent| accent.key == key)
}

/// What every widget draws with, as `prefs` asks for it.
fn palette_for(prefs: &Preferences, truecolor: bool) -> Palette {
    let mut palette = resolve(&prefs.selected_theme, &prefs.selected_variant);
    if prefs.high_contrast {
        palette = palette.with_high_contrast();
    }
    if let Some(accent) = picked_accent(prefs) {
        palette = palette.with_accent(accent.color);
    }
    if truecolor {
        palette
    } else {
        palette.map(nearest_256)
    }
}

static ACTIVE: RwLock<Option<Palette>> = RwLock::new(None);

/// Applies the theme, accent and contrast in `prefs`, as colony-ui's
/// `StandardPreferences::apply` does: every frame drawn after this uses them.
pub fn apply(prefs: &Preferences) {
    let palette = palette_for(prefs, truecolor());
    *ACTIVE.write().unwrap_or_else(PoisonError::into_inner) = Some(palette);
}

/// The palette in effect: the defaults until [`apply`] is called.
pub fn active() -> Palette {
    let active = *ACTIVE.read().unwrap_or_else(PoisonError::into_inner);
    active.unwrap_or_else(|| palette_for(&Preferences::default(), truecolor()))
}

/// `color` as this terminal can show it.
pub fn paint(color: Color) -> Color {
    if truecolor() {
        color
    } else {
        nearest_256(color)
    }
}

/// Whether the terminal says it shows 24-bit colour. Windows Terminal does
/// but does not say so in COLORTERM, so its own variable counts too.
fn truecolor() -> bool {
    static TRUECOLOR: OnceLock<bool> = OnceLock::new();
    *TRUECOLOR.get_or_init(|| {
        std::env::var_os("WT_SESSION").is_some()
            || std::env::var("COLORTERM")
                .is_ok_and(|value| value == "truecolor" || value == "24bit")
    })
}

/// The closest of the xterm 256 colours that is not one of the first 16,
/// which every terminal redefines: the 6x6x6 cube or the 24-step grey ramp.
fn nearest_256(color: Color) -> Color {
    let Color::Rgb(r, g, b) = color else {
        return color;
    };
    const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
    let level = |c: u8| (0..6).min_by_key(|&i| LEVELS[i].abs_diff(c)).unwrap_or(0);
    let (ri, gi, bi) = (level(r), level(g), level(b));
    let average = (u16::from(r) + u16::from(g) + u16::from(b)) / 3;
    let grey = (average.saturating_sub(3) / 10).min(23) as u8;
    let distance = |(cr, cg, cb): (u8, u8, u8)| {
        [(r, cr), (g, cg), (b, cb)]
            .map(|(a, b)| u32::from(a.abs_diff(b)).pow(2))
            .iter()
            .sum::<u32>()
    };
    let grey_value = 8 + 10 * grey;
    let cube = (LEVELS[ri], LEVELS[gi], LEVELS[bi]);
    if distance((grey_value, grey_value, grey_value)) < distance(cube) {
        Color::Indexed(232 + grey)
    } else {
        Color::Indexed(16 + 36 * ri as u8 + 6 * gi as u8 + bi as u8)
    }
}

/// `color` moved `amount` (-1 to 1) on each channel, as colony-ui moves its
/// float colours, then rounded back to a byte.
fn shift(color: Color, amount: f32) -> Color {
    let Color::Rgb(r, g, b) = color else {
        return color;
    };
    let channel = |c: u8| {
        let c = f32::from(c) / 255.0;
        let moved = if amount < 0.0 {
            (c + amount).max(0.0)
        } else {
            (c + amount).min(1.0)
        };
        (moved * 255.0).round() as u8
    };
    Color::Rgb(channel(r), channel(g), channel(b))
}

fn channels(color: Color) -> [f32; 3] {
    match color {
        Color::Rgb(r, g, b) => [r, g, b].map(|c| f32::from(c) / 255.0),
        _ => [0.0; 3],
    }
}

/// colony-ui's `ColorExt::is_light`: YIQ luma over one half.
fn is_light(color: Color) -> bool {
    let [r, g, b] = channels(color);
    0.299 * r + 0.587 * g + 0.114 * b > 0.5
}

/// colony-ui's `contrast_on`: the ink or the paper of `contrastEnds`,
/// whichever reads better on `background` by WCAG contrast.
pub fn contrast_on(background: Color) -> Color {
    readable_end(background, catalog().contrast_ends)
}

/// [`contrast_on`] with the ends given, for the catalog still being read.
fn readable_end(background: Color, [ink, paper]: ContrastEnds) -> Color {
    if contrast_ratio(ink, background) >= contrast_ratio(paper, background) {
        ink
    } else {
        paper
    }
}

/// WCAG contrast ratio, 1 to 21.
fn contrast_ratio(a: Color, b: Color) -> f32 {
    let luminance = |color| {
        let [r, g, b] = channels(color).map(|v| {
            if v <= 0.03928 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        });
        0.2126 * r + 0.7152 * g + 0.0722 * b
    };
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn every_family_variant_and_accent_loads_with_both_labels() {
        let json: Value = serde_json::from_str(THEMES_JSON).unwrap();
        let catalog = catalog();
        let count = |key| json[key].as_u64().unwrap() as usize;
        assert_eq!(catalog.families.len(), count("familyCount"));
        let variants: usize = catalog.families.iter().map(|f| f.variants.len()).sum();
        assert_eq!(variants, count("variantCount"), "a variant was skipped");
        assert_eq!(catalog.accents.len(), 8);

        let labels = catalog.families.iter().flat_map(|family| {
            std::iter::once(&family.label).chain(family.variants.iter().map(|v| &v.label))
        });
        for label in labels.chain(catalog.accents.iter().map(|a| &a.label)) {
            assert!(!label.en.trim().is_empty(), "{label:?}");
            assert!(!label.fr.trim().is_empty(), "{label:?}");
        }
    }

    #[test]
    fn an_unknown_theme_falls_back_to_gruvbox_dark() {
        let gruvbox_dark = resolve("gruvbox", "dark");
        assert_eq!(gruvbox_dark.bg_primary, Color::Rgb(0x28, 0x28, 0x28));
        assert_eq!(resolve("no_such_family", "dark"), gruvbox_dark);
        assert_eq!(resolve("gruvbox", "no_such_variant"), gruvbox_dark);
        assert_ne!(resolve("catppuccin", "latte"), gruvbox_dark);

        let prefs = Preferences {
            selected_theme: "removed".into(),
            ..Preferences::default()
        };
        assert_eq!(palette_for(&prefs, true), gruvbox_dark);
    }

    /// colony-ui's float arithmetic, rounded to the nearest byte. Worked out
    /// by hand from the gruvbox tokens, not by running this code.
    #[test]
    fn high_contrast_moves_text_and_borders_as_colony_ui_does() {
        let light = resolve("gruvbox", "light").with_high_contrast();
        assert_eq!(light.text_primary, Color::Rgb(22, 18, 16));
        assert_eq!(light.text_secondary, Color::Rgb(49, 42, 38));
        assert_eq!(light.text_muted, Color::Rgb(77, 67, 59));
        assert_eq!(light.text_dim, Color::Rgb(99, 86, 75));
        assert_eq!(light.border_subtle, Color::Rgb(175, 158, 123));
        assert_eq!(light.divider, Color::Rgb(175, 158, 123));

        let dark = resolve("gruvbox", "dark").with_high_contrast();
        assert_eq!(dark.text_primary, Color::Rgb(255, 250, 209));
        assert_eq!(dark.text_secondary, Color::Rgb(239, 222, 187));
        assert_eq!(dark.text_muted, Color::Rgb(215, 200, 173));
        assert_eq!(dark.text_dim, Color::Rgb(188, 173, 152));
        assert_eq!(dark.border_subtle, Color::Rgb(91, 87, 85));

        // The surface itself never moves.
        assert_eq!(dark.bg_primary, resolve("gruvbox", "dark").bg_primary);
    }

    #[test]
    fn a_picked_accent_wins_unless_auto_accent_is_on() {
        let theme_accent = resolve("gruvbox", "dark").accent;
        let mut prefs = Preferences {
            selected_accent: Some("violet".into()),
            ..Preferences::default()
        };
        let violet = palette_for(&prefs, true);
        assert_eq!(violet.accent, Color::Rgb(0xb0, 0x6b, 0xd6));
        assert_eq!(violet.on_accent, contrast_on(violet.accent));

        prefs.auto_accent = true;
        assert_eq!(palette_for(&prefs, true).accent, theme_accent);
        prefs.auto_accent = false;
        prefs.selected_accent = Some("chartreuse".into());
        assert_eq!(palette_for(&prefs, true).accent, theme_accent);
    }

    #[test]
    fn text_on_a_fill_is_whichever_end_reads_better() {
        // colony-ui's INK (0.08, 0.08, 0.10) and PAPER (0.97, 0.98, 1.0).
        let (ink, paper) = (Color::Rgb(20, 20, 26), Color::Rgb(247, 250, 255));
        assert_eq!(
            catalog().contrast_ends,
            [ink, paper],
            "contrastEnds left colony-ui's ends"
        );

        assert_eq!(contrast_on(Color::Rgb(255, 255, 255)), ink);
        assert_eq!(contrast_on(Color::Rgb(0, 0, 0)), paper);
        // colony-ui's own example: green takes ink, not paper.
        assert_eq!(contrast_on(Color::Rgb(0x55, 0xb8, 0x7a)), ink);
    }

    #[test]
    fn colours_map_to_the_nearest_of_the_256() {
        assert_eq!(nearest_256(Color::Rgb(0, 0, 0)), Color::Indexed(16));
        assert_eq!(nearest_256(Color::Rgb(255, 255, 255)), Color::Indexed(231));
        assert_eq!(nearest_256(Color::Rgb(255, 0, 0)), Color::Indexed(196));
        // Greys land on the ramp, which is finer than the cube's diagonal.
        assert_eq!(nearest_256(Color::Rgb(128, 128, 128)), Color::Indexed(244));
        assert_eq!(
            nearest_256(Color::Rgb(0x28, 0x28, 0x28)),
            Color::Indexed(235)
        );
        // Gruvbox's accent, blue-grey: the cube.
        assert_eq!(
            nearest_256(Color::Rgb(0x83, 0xa5, 0x98)),
            Color::Indexed(108)
        );
        assert_eq!(nearest_256(Color::Indexed(3)), Color::Indexed(3));

        let mapped = palette_for(&Preferences::default(), false);
        assert!(matches!(mapped.bg_primary, Color::Indexed(_)));
        assert!(matches!(mapped.on_accent, Color::Indexed(_)));
    }

    /// Every colour comes from the palette (design/settings-page.md): no
    /// other source file spells one out.
    #[test]
    fn no_colour_is_written_outside_this_file() {
        let mut dirs = vec![PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src"))];
        let mut found = Vec::new();
        while let Some(dir) = dirs.pop() {
            for entry in fs::read_dir(&dir).expect("read src") {
                let path = entry.expect("read src").path();
                if path.is_dir() {
                    dirs.push(path);
                } else if !path.ends_with("theme.rs")
                    && fs::read_to_string(&path).expect("read").contains("Color::")
                {
                    found.push(path.display().to_string());
                }
            }
        }
        assert!(found.is_empty(), "colours outside src/theme.rs: {found:?}");
    }
}
