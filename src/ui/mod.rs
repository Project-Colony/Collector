//! Drawing: a header bar, the monitor's boxes (one file each) or the
//! Preferences page in their place, and a footer.

mod cpu_memory;
mod disks;
mod network;
pub mod preferences;
mod processes;
mod style;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

use crate::app::{App, Message};
use crate::i18n::t;
use crate::paths;

pub use preferences::{Focus, Page};

/// Where a click lands, and what it does: the same message as the key that
/// does the same thing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hit {
    pub area: Rect,
    pub message: Message,
}

/// Draws the screen and returns what can be clicked on it.
pub fn render_ui(frame: &mut Frame, app: &App) -> Vec<Hit> {
    let mut hits = Vec::new();
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .horizontal_margin(1)
    .areas(frame.area());

    // The program's name is the way into Preferences, and out again
    // (design/navigation.md): one button, marked while the page is open.
    let name = format!(" {} ", paths::PROGRAM);
    let button = Rect {
        width: (Span::raw(name.as_str()).width() as u16).min(header.width),
        ..header
    };
    let open = app.page.is_some();
    let style = style::item(open, hovered(app, button)).patch(style::title());
    frame.render_widget(Paragraph::new(name).style(style), button);
    hits.push(Hit {
        area: button,
        message: Message::TogglePreferences,
    });

    match &app.page {
        Some(page) => preferences::render(frame, app, page, body, &mut hits),
        None => render_monitor(frame, app, body),
    }

    // Full width, where the French hints fit too. A status message about the
    // preferences file takes the line until the next key or click.
    let footer_text = match (&app.status, open) {
        (Some(status), _) => Paragraph::new(status.as_str()),
        (None, false) => Paragraph::new(t("monitor_hint")).style(style::muted()),
        (None, true) => Paragraph::new(t("preferences_hint")).style(style::muted()),
    };
    frame.render_widget(footer_text, footer);
    hits
}

/// Whether the mouse is over `area`.
fn hovered(app: &App, area: Rect) -> bool {
    app.mouse.is_some_and(|position| area.contains(position))
}

fn render_monitor(frame: &mut Frame, app: &App, area: Rect) {
    // The processes take what is left, so a short terminal shrinks the list
    // and never the boxes above it.
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(7),
            Constraint::Fill(1),
        ])
        .split(area);

    cpu_memory::render(frame, app, main_layout[0]);

    let disk_network = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_layout[1]);
    disks::render(frame, app, disk_network[0]);
    network::render(frame, app, disk_network[1]);

    processes::render(frame, app, main_layout[2]);
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;

    use super::*;
    use crate::i18n::{self, Locale, number, tf};

    /// The buffer as text, one line per row.
    fn screen_text(buffer: &Buffer) -> String {
        let width = usize::from(buffer.area.width);
        buffer
            .content()
            .chunks(width)
            .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Draws `app` at 80x24, the smallest usual terminal size.
    fn draw(app: &App) -> (String, Vec<Hit>) {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).expect("test terminal");
        let mut hits = Vec::new();
        terminal
            .draw(|frame| hits = render_ui(frame, app))
            .expect("draw");
        (screen_text(terminal.backend().buffer()), hits)
    }

    fn assert_shown(screen: &str, locale: Locale, expected: &[String]) {
        for text in expected {
            assert!(
                screen.contains(text.as_str()),
                "{locale:?}: {text:?} is cut off or missing:\n{screen}"
            );
        }
        // A key with no string would show the key itself.
        for prefix in ["settings_", "preferences_"] {
            assert!(!screen.contains(prefix), "{locale:?}: a raw key:\n{screen}");
        }
    }

    /// French runs 15 to 20% longer than English: every title, header and
    /// hint is drawn whole in both.
    #[test]
    fn every_label_fits_at_80x24_in_both_locales() {
        let app = App::empty();
        for locale in [Locale::En, Locale::Fr] {
            let (screen, expected) = i18n::with_locale(locale, || {
                let zero = number(0.0, 1);
                let mut expected: Vec<String> = [
                    "cpu",
                    "memory",
                    "disks",
                    "network",
                    "top_processes",
                    "process",
                    "monitor_hint",
                ]
                .map(|key| t(key).to_owned())
                .into();
                expected.push(tf("rx_rate", &[("rate", &zero)]));
                expected.push(tf("tx_rate", &[("rate", &zero)]));
                expected.push(tf("used_of_total_gb", &[("used", &zero), ("total", &zero)]));
                expected.push(paths::PROGRAM.to_owned());
                (draw(&app).0, expected)
            });
            assert_shown(&screen, locale, &expected);
        }
    }

    /// Each category with every section open: the page header, every
    /// category name, every heading, section and setting are drawn whole in
    /// both locales.
    #[test]
    fn the_preferences_page_fits_at_80x24_in_both_locales() {
        for locale in [Locale::En, Locale::Fr] {
            for category in 0..preferences::CATEGORIES.len() {
                let mut app = App::empty();
                app.update(Message::TogglePreferences);
                app.update(Message::Category(category));
                let page = app.page.as_mut().expect("open");
                page.expanded.extend(["language", "refresh"]);

                let (screen, expected) = i18n::with_locale(locale, || {
                    let page = app.page.as_ref().expect("open");
                    let mut expected: Vec<String> =
                        ["settings_title", "settings_close", "preferences_hint"]
                            .into_iter()
                            .chain(preferences::CATEGORIES)
                            .map(|key| t(key).to_owned())
                            .collect();
                    for row in preferences::rows(&app.prefs, page) {
                        match row {
                            preferences::Row::Section { title, .. } => expected.push(title.into()),
                            preferences::Row::Setting { title, value, .. } => {
                                expected.push(title.into());
                                expected.push(value);
                            }
                            // Wrapped: each word is there.
                            preferences::Row::Note(text) => {
                                expected.extend(text.split(' ').map(str::to_owned));
                            }
                            _ => {}
                        }
                    }
                    (draw(&app).0, expected)
                });
                assert_shown(&screen, locale, &expected);
            }
        }
    }

    /// The identity button and the page's Close button send the same
    /// message, and that message closes what it opened.
    #[test]
    fn the_identity_button_and_close_send_the_same_toggle() {
        let mut app = App::empty();
        let (_, hits) = draw(&app);
        let identity = hits[0];
        assert_eq!(identity.message, Message::TogglePreferences);
        assert_eq!(identity.area.y, 0, "top-left");

        app.update(identity.message);
        assert!(app.page.is_some());
        let (_, hits) = draw(&app);
        let toggles: Vec<_> = hits
            .iter()
            .filter(|hit| hit.message == Message::TogglePreferences)
            .collect();
        assert_eq!(toggles.len(), 2, "identity and Close: {hits:?}");
        assert!(toggles.iter().any(|hit| hit.area == identity.area));

        app.update(toggles[1].message);
        assert!(app.page.is_none());
    }
}
