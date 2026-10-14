//! The Preferences page. It replaces the monitor's boxes, not the screen: the
//! header bar and the footer stay (design/settings-page.md). Its header, its
//! category list on the left and its collapsible sections follow the
//! colony-ui kit, drawn in cells instead of iced widgets.

use std::collections::HashSet;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{Hit, hovered, style};
use crate::app::{App, Message};
use crate::i18n::{number, t, tf};
use crate::preferences::{self, Language, Preferences, Setting};
use crate::theme;

/// colony-ui's `STANDARD_CATEGORIES` in their fixed order, then its
/// `ABOUT_CATEGORY`, last.
pub const CATEGORIES: [&str; 4] = [
    "settings_cat_general",
    "settings_cat_appearance",
    "settings_cat_accessibility",
    "settings_cat_about",
];

/// The line under each category's heading.
const DESCRIPTIONS: [&str; 4] = [
    "settings_general_desc",
    "settings_appearance_desc",
    "settings_accessibility_desc",
    "preferences_about_desc",
];

/// Section chevrons, from the WGL4 set every console font has: no Nerd Font
/// needed.
const COLLAPSED: &str = "►";
const EXPANDED: &str = "▼";
/// The mark on the current theme card and accent swatch. Not WGL4, but in
/// every common monospace font, and a terminal falls back for the others.
const CHECK: &str = "✓";
/// Where the cards and swatches start: under their row's title.
const INDENT: &str = "    ";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Focus {
    /// Up and Down change the category.
    #[default]
    Categories,
    /// Up and Down move between the category's sections and settings.
    Content,
}

/// What the page remembers while it is open. Closing drops it, so the page
/// always opens on General with every section closed.
#[derive(Debug, Default)]
pub struct Page {
    pub category: usize,
    pub focus: Focus,
    /// The focused row, counted among the rows that take focus.
    pub row: usize,
    /// The open sections, by key.
    pub expanded: HashSet<&'static str>,
}

/// One entry of a category, below its heading and description.
pub enum Row {
    /// A collapsible section header. Enter or a click opens or closes it.
    Section {
        key: &'static str,
        title: &'static str,
        expanded: bool,
    },
    /// A setting and its current value. Left and Right change it; Enter or a
    /// click moves it to the next value.
    Setting {
        setting: Setting,
        title: &'static str,
        value: String,
    },
    /// A muted explanation.
    Note(String),
    Text(String),
    /// A blank line after an open section.
    Gap,
    /// A theme family's variant cards, under its row. Each card is a click
    /// target of its own.
    Cards(usize),
    /// The accent swatches, under the accent row, clicked one by one too.
    Swatches,
    /// A live sample of the current theme and accent.
    Preview,
}

impl Row {
    pub fn takes_focus(&self) -> bool {
        matches!(self, Self::Section { .. } | Self::Setting { .. })
    }
}

/// The open category's rows, top to bottom.
pub fn rows(prefs: &Preferences, page: &Page) -> Vec<Row> {
    let mut rows = Vec::new();
    match page.category {
        0 => {
            section(
                &mut rows,
                page,
                "language",
                "preferences_section_language",
                || {
                    vec![
                        Row::Note(t("preferences_language_desc").to_owned()),
                        Row::Setting {
                            setting: Setting::Language,
                            title: t("preferences_language"),
                            value: language_label(prefs.language).to_owned(),
                        },
                    ]
                },
            );
            section(
                &mut rows,
                page,
                "refresh",
                "preferences_section_refresh",
                || {
                    vec![
                        Row::Note(t("preferences_refresh_desc").to_owned()),
                        Row::Setting {
                            setting: Setting::RefreshInterval,
                            title: t("preferences_refresh_interval"),
                            value: seconds(prefs.refresh_interval),
                        },
                    ]
                },
            );
        }
        3 => {
            let version = env!("CARGO_PKG_VERSION");
            rows.push(Row::Text(tf(
                "preferences_version",
                &[("version", version)],
            )));
            let license = env!("CARGO_PKG_LICENSE");
            rows.push(Row::Text(tf(
                "preferences_license",
                &[("license", license)],
            )));
            let url = env!("CARGO_PKG_REPOSITORY");
            rows.push(Row::Text(tf("preferences_source", &[("url", url)])));
            rows.push(Row::Gap);
            rows.push(Row::Note(t("preferences_file").to_owned()));
            rows.push(Row::Text(match preferences::path() {
                Ok(path) => path.display().to_string(),
                Err(err) => err.to_string(),
            }));
        }
        1 => {
            section(&mut rows, page, "theme", "settings_section_theme", || {
                let mut rows = vec![Row::Note(t("settings_theme_desc").to_owned())];
                // Without the family's glyph: it is a Nerd Font one, which
                // the terminal's font may not have.
                for (index, family) in theme::catalog().families.iter().enumerate() {
                    let current = family.variants.iter().find(|variant| {
                        family.key == prefs.selected_theme && variant.key == prefs.selected_variant
                    });
                    rows.push(Row::Setting {
                        setting: Setting::Theme(index),
                        title: family.label.text(),
                        value: current.map_or_else(String::new, |v| v.label.text().to_owned()),
                    });
                    rows.push(Row::Cards(index));
                }
                rows
            });
            section(&mut rows, page, "colors", "settings_section_colors", || {
                let accent = theme::picked_accent(prefs)
                    .map_or(t("settings_default_accent"), |accent| accent.label.text());
                vec![
                    Row::Note(t("settings_colors_desc").to_owned()),
                    Row::Setting {
                        setting: Setting::Accent,
                        title: t("preferences_accent"),
                        value: accent.to_owned(),
                    },
                    Row::Swatches,
                    Row::Setting {
                        setting: Setting::AutoAccent,
                        title: t("settings_auto_accent"),
                        value: on_off(prefs.auto_accent),
                    },
                    Row::Note(t("settings_auto_accent_desc").to_owned()),
                ]
            });
            section(
                &mut rows,
                page,
                "preview",
                "settings_section_preview",
                || {
                    vec![
                        Row::Note(t("settings_preview_desc").to_owned()),
                        Row::Preview,
                    ]
                },
            );
            closing_note(&mut rows, "preferences_appearance_note");
        }
        2 => {
            section(&mut rows, page, "vision", "settings_section_vision", || {
                vec![
                    Row::Note(t("settings_vision_desc").to_owned()),
                    Row::Setting {
                        setting: Setting::HighContrast,
                        title: t("settings_high_contrast"),
                        value: on_off(prefs.high_contrast),
                    },
                    Row::Note(t("settings_high_contrast_desc").to_owned()),
                    Row::Note(t("preferences_font_note").to_owned()),
                ]
            });
            section(&mut rows, page, "motion", "settings_section_motion", || {
                vec![
                    Row::Note(t("settings_motion_desc").to_owned()),
                    Row::Setting {
                        setting: Setting::ReduceMotion,
                        title: t("settings_reduce_motion"),
                        value: on_off(prefs.reduce_motion),
                    },
                    Row::Note(t("settings_reduce_motion_desc").to_owned()),
                    Row::Note(t("preferences_motion_note").to_owned()),
                ]
            });
            closing_note(&mut rows, "preferences_reading_note");
        }
        _ => {}
    }
    rows
}

/// What a category leaves out and why, a blank line below its sections.
fn closing_note(rows: &mut Vec<Row>, key: &'static str) {
    if !matches!(rows.last(), Some(Row::Gap)) {
        rows.push(Row::Gap);
    }
    rows.push(Row::Note(t(key).to_owned()));
}

fn on_off(on: bool) -> String {
    t(if on {
        "preferences_on"
    } else {
        "preferences_off"
    })
    .to_owned()
}

/// A section header, and its content when it is open.
fn section(
    rows: &mut Vec<Row>,
    page: &Page,
    key: &'static str,
    title: &'static str,
    content: impl FnOnce() -> Vec<Row>,
) {
    let expanded = page.expanded.contains(key);
    rows.push(Row::Section {
        key,
        title: t(title),
        expanded,
    });
    if expanded {
        rows.extend(content());
        rows.push(Row::Gap);
    }
}

fn language_label(language: Language) -> &'static str {
    match language {
        Language::System => t("preferences_language_system"),
        Language::En => t("language_en"),
        Language::Fr => t("language_fr"),
    }
}

/// `0.5 s`, `1 s`: no decimal part when there is none.
fn seconds(value: f64) -> String {
    let decimals = if value.fract() == 0.0 { 0 } else { 1 };
    tf("seconds", &[("value", &number(value, decimals))])
}

pub fn render(frame: &mut Frame, app: &App, page: &Page, area: Rect, hits: &mut Vec<Hit>) {
    let [header, body] = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

    // The title, then Close at the far right. Close sends the identity
    // button's message: two ways out, one message.
    let close = format!(" {} ", t("settings_close"));
    let close_width = Span::raw(close.as_str()).width() as u16;
    let [title, close_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(close_width)]).areas(header);
    frame.render_widget(
        Paragraph::new(t("settings_title")).style(style::title()),
        title,
    );
    let close_style = style::muted().patch(style::item(false, hovered(app, close_area)));
    frame.render_widget(Paragraph::new(close).style(close_style), close_area);
    hits.push(Hit {
        area: close_area,
        message: Message::TogglePreferences,
    });

    let labels = CATEGORIES.map(t);
    // As wide as the longest name in this locale, plus a space each side and
    // the border: the list grows with the language rather than cutting it.
    let list_width = labels
        .iter()
        .map(|label| Span::raw(*label).width())
        .max()
        .unwrap_or(0)
        + 3;
    let [list, content] =
        Layout::horizontal([Constraint::Length(list_width as u16), Constraint::Fill(1)])
            .spacing(1)
            .areas(body);

    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(style::divider());
    let inner = block.inner(list);
    frame.render_widget(block, list);
    for (index, (label, y)) in labels.iter().zip(inner.y..inner.bottom()).enumerate() {
        let item = Rect {
            y,
            height: 1,
            ..inner
        };
        // Muted unless selected, which the accent's on_accent text overrides.
        let style = style::muted().patch(style::item(index == page.category, hovered(app, item)));
        frame.render_widget(Paragraph::new(format!(" {label}")).style(style), item);
        hits.push(Hit {
            area: item,
            message: Message::Category(index),
        });
    }

    render_content(frame, app, page, content, hits);
}

fn render_content(frame: &mut Frame, app: &App, page: &Page, area: Rect, hits: &mut Vec<Hit>) {
    let width = usize::from(area.width);
    let prefs = &app.prefs;
    let mut lines = vec![Line::styled(t(CATEGORIES[page.category]), style::title())];
    lines.extend(muted_lines(t(DESCRIPTIONS[page.category]), width));
    lines.push(Line::default());

    // (line, index among the rows that take focus)
    let mut targets = Vec::new();
    let mut picks = Vec::new();
    for row in rows(prefs, page) {
        if row.takes_focus() {
            targets.push((lines.len(), targets.len()));
        }
        match row {
            Row::Section {
                title, expanded, ..
            } => {
                let chevron = if expanded { EXPANDED } else { COLLAPSED };
                lines.push(spread(
                    Span::styled(title, style::title()),
                    Span::raw(chevron),
                    width,
                ));
            }
            Row::Setting { title, value, .. } => {
                let title = Span::raw(format!("  {title}"));
                lines.push(if value.is_empty() {
                    Line::from(title)
                } else {
                    let value = Span::styled(format!("‹ {value} ›"), style::secondary());
                    spread(title, value, width)
                });
            }
            Row::Note(text) => lines.extend(muted_lines(&text, width)),
            Row::Text(text) => lines.extend(wrap(&text, width).into_iter().map(Line::from)),
            Row::Gap => lines.push(Line::default()),
            Row::Cards(family) => cards(prefs, family, width, &mut lines, &mut picks),
            Row::Swatches => swatches(prefs, &mut lines, &mut picks),
            Row::Preview => lines.extend(preview(prefs)),
        }
    }

    // Scrolled just enough to keep the focused row on screen, with what
    // belongs to it up to the next row (its cards, its description) when
    // that fits too.
    let focused = |index: usize| page.focus == Focus::Content && index == page.row;
    let (focused_line, end) = targets
        .iter()
        .position(|&(_, index)| focused(index))
        .map_or((0, 0), |at| {
            let next = targets.get(at + 1).map_or(lines.len(), |&(line, _)| line);
            (targets[at].0, next)
        });
    let offset = end
        .saturating_sub(usize::from(area.height))
        .min(focused_line);
    let screen_row = |line: usize| {
        line.checked_sub(offset)
            .and_then(|row| u16::try_from(row).ok())
            .filter(|&row| row < area.height)
            .map(|row| area.y + row)
    };

    for (line, index) in targets {
        let Some(y) = screen_row(line) else {
            continue;
        };
        let target = Rect {
            y,
            height: 1,
            ..area
        };
        // Over the spans' own colours too, so selected text always sits on
        // the accent in the accent's own legible colour.
        let style = style::item(focused(index), hovered(app, target));
        let line = &mut lines[line];
        line.style = line.style.patch(style);
        for span in &mut line.spans {
            span.style = span.style.patch(style);
        }
        hits.push(Hit {
            area: target,
            message: Message::Activate(index),
        });
    }
    for pick in picks {
        let Some(y) = screen_row(pick.line) else {
            continue;
        };
        let x = area
            .x
            .saturating_add(u16::try_from(pick.x).unwrap_or(u16::MAX));
        let width = u16::try_from(pick.width).unwrap_or(u16::MAX);
        let target = Rect::new(x, y, width, 1).intersection(area);
        if !target.is_empty() {
            hits.push(Hit {
                area: target,
                message: pick.message,
            });
        }
    }

    frame.render_widget(Paragraph::new(lines.split_off(offset)), area);
}

/// A click target inside a line: a theme card or an accent swatch.
struct Pick {
    line: usize,
    /// Columns from the left of the content.
    x: usize,
    width: usize,
    message: Message,
}

/// A family's variant cards, as many to a line as fit. A card is the
/// variant's background with a bar of its accent, then its name, and a check
/// on the current one, as colony-ui's theme picker draws them.
fn cards(
    prefs: &Preferences,
    family: usize,
    width: usize,
    lines: &mut Vec<Line<'static>>,
    picks: &mut Vec<Pick>,
) {
    let Some(meta) = theme::catalog().families.get(family) else {
        return;
    };
    let mut line = Line::from(INDENT);
    for (index, variant) in meta.variants.iter().enumerate() {
        let current = meta.key == prefs.selected_theme && variant.key == prefs.selected_variant;
        let mark = if current { CHECK } else { " " };
        let name = format!("{} {mark} ", variant.label.text());
        let card = 1 + Span::raw(name.as_str()).width();
        if line.width() > INDENT.len() {
            if line.width() + 1 + card > width {
                lines.push(std::mem::replace(&mut line, Line::from(INDENT)));
            } else {
                line.push_span(" ");
            }
        }
        picks.push(Pick {
            line: lines.len(),
            x: line.width(),
            width: card,
            message: Message::Theme {
                family,
                variant: index,
            },
        });
        // Painted from the variant's own colours, not the active theme's:
        // each card shows what it would look like.
        let background = theme::paint(variant.swatch_bg);
        let on = |color| Style::new().fg(theme::paint(color)).bg(background);
        line.push_span(Span::styled("▌", on(variant.swatch_accent)));
        line.push_span(Span::styled(
            name,
            on(theme::contrast_on(variant.swatch_bg)),
        ));
    }
    lines.push(line);
}

/// The shared accents as filled circles, a check after the picked one.
fn swatches(prefs: &Preferences, lines: &mut Vec<Line<'static>>, picks: &mut Vec<Pick>) {
    let picked = theme::picked_accent(prefs).map(|accent| accent.key.as_str());
    let mut line = Line::from(INDENT);
    for (index, accent) in theme::catalog().accents.iter().enumerate() {
        picks.push(Pick {
            line: lines.len(),
            x: line.width(),
            width: 2,
            message: Message::Accent(index),
        });
        let dot = Style::new().fg(theme::paint(accent.color));
        line.push_span(Span::styled("●", dot));
        line.push_span(if picked == Some(accent.key.as_str()) {
            CHECK
        } else {
            " "
        });
        line.push_span(" ");
    }
    lines.push(line);
}

/// colony-ui's Preview card in cells: a title, body text and the theme and
/// accent names down the text ramp, then a gauge and a button in the accent.
fn preview(prefs: &Preferences) -> Vec<Line<'static>> {
    let family = theme::catalog()
        .families
        .iter()
        .find(|family| family.key == prefs.selected_theme);
    let theme_name = family.map_or_else(
        || prefs.selected_theme.clone(),
        |family| {
            let variant = family
                .variants
                .iter()
                .find(|variant| variant.key == prefs.selected_variant)
                .map_or("", |variant| variant.label.text());
            format!("{} · {variant}", family.label.text())
        },
    );
    let accent = theme::picked_accent(prefs)
        .map_or(t("settings_default_accent"), |accent| accent.label.text());
    let card = |content: Vec<Span<'static>>| {
        let mut line = Line::from(Span::styled("│ ", style::divider()));
        line.spans.extend(content);
        line
    };
    vec![
        card(vec![Span::styled(
            t("settings_preview_card"),
            style::title(),
        )]),
        card(vec![Span::styled(
            t("settings_preview_body"),
            style::secondary(),
        )]),
        card(vec![Span::styled(
            format!("{theme_name} · {accent}"),
            style::muted(),
        )]),
        card(vec![
            Span::styled("█".repeat(12), style::gauge()),
            Span::styled(" ".repeat(8), style::gauge()),
            Span::raw("  "),
            Span::styled(
                format!(" {} ", t("settings_preview_button")),
                style::item(true, false),
            ),
        ]),
    ]
}

/// `left` at the start of the line and `right` at its end.
fn spread<'a>(left: Span<'a>, right: Span<'a>, width: usize) -> Line<'a> {
    let gap = width.saturating_sub(left.width() + right.width()).max(1);
    Line::from(vec![left, Span::raw(" ".repeat(gap)), right])
}

fn muted_lines(text: &str, width: usize) -> Vec<Line<'static>> {
    wrap(text, width)
        .into_iter()
        .map(|line| Line::styled(line, style::muted()))
        .collect()
}

/// Breaks `text` between words to fit `width` columns, and a word longer than
/// a line (a path) wherever it has to.
// ponytail: counts characters, not display columns, so a wide (CJK) character
// in a path can push a line past the edge, where it is cut. Measure with
// unicode-width if that ever shows up.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut lines = Vec::new();
    let mut line = String::new();
    for mut word in text.split(' ') {
        let used = line.chars().count();
        let needed = word.chars().count() + usize::from(used > 0);
        if used + needed <= width {
            if used > 0 {
                line.push(' ');
            }
            line.push_str(word);
            continue;
        }
        if used > 0 {
            lines.push(std::mem::take(&mut line));
        }
        while word.chars().count() > width {
            let cut = word
                .char_indices()
                .nth(width)
                .map_or(word.len(), |(at, _)| at);
            lines.push(word[..cut].to_owned());
            word = &word[cut..];
        }
        line.push_str(word);
    }
    lines.push(line);
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_standard_categories_come_first_and_about_last() {
        assert_eq!(
            CATEGORIES,
            [
                "settings_cat_general",
                "settings_cat_appearance",
                "settings_cat_accessibility",
                "settings_cat_about",
            ]
        );
    }

    #[test]
    fn wrapping_keeps_words_whole_and_cuts_only_what_cannot_fit() {
        assert_eq!(wrap("one two three", 7), ["one two", "three"]);
        assert_eq!(wrap("short", 20), ["short"]);
        assert_eq!(wrap("/a/very/long/path", 6), ["/a/ver", "y/long", "/path"]);
        assert_eq!(wrap("Réseau é", 6), ["Réseau", "é"]);
    }

    #[test]
    fn sections_arrive_closed_and_show_their_content_open() {
        let prefs = Preferences::default();
        let mut page = Page::default();
        let focusable = |page: &Page| {
            rows(&prefs, page)
                .iter()
                .filter(|r| r.takes_focus())
                .count()
        };
        assert_eq!(focusable(&page), 2, "two closed sections");
        page.expanded.insert("language");
        assert_eq!(focusable(&page), 3, "the language setting shows");
    }
}
