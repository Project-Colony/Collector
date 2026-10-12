//! Drawing: one file per box.

mod cpu_memory;
mod disks;
mod network;
mod processes;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::Paragraph;

use crate::app::App;
use crate::i18n::t;

pub fn render_ui(frame: &mut Frame, app: &App) {
    // The processes take what is left, so a short terminal shrinks the list
    // and never the boxes above it or the footer.
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(7),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    cpu_memory::render(frame, app, main_layout[0]);

    let disk_network = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_layout[1]);
    disks::render(frame, app, disk_network[0]);
    network::render(frame, app, disk_network[1]);

    processes::render(frame, app, main_layout[2]);

    // Full width, where the French hint fits too.
    frame.render_widget(Paragraph::new(t("quit_hint")), main_layout[3]);
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

    /// French runs 15 to 20% longer than English: at the smallest usual
    /// terminal size, every title, header and hint is drawn whole in both.
    #[test]
    fn every_label_fits_at_80x24_in_both_locales() {
        let app = App::empty();
        for locale in [Locale::En, Locale::Fr] {
            let (screen, expected) = i18n::with_locale(locale, || {
                let mut terminal = Terminal::new(TestBackend::new(80, 24)).expect("test terminal");
                terminal.draw(|frame| render_ui(frame, &app)).expect("draw");
                let zero = number(0.0, 1);
                let mut expected: Vec<String> = [
                    "cpu",
                    "memory",
                    "disks",
                    "network",
                    "top_processes",
                    "process",
                    "quit_hint",
                ]
                .map(|key| t(key).to_owned())
                .into();
                expected.push(tf("rx_rate", &[("rate", &zero)]));
                expected.push(tf("tx_rate", &[("rate", &zero)]));
                expected.push(tf("used_of_total_gb", &[("used", &zero), ("total", &zero)]));
                (screen_text(terminal.backend().buffer()), expected)
            });
            for text in expected {
                assert!(
                    screen.contains(&text),
                    "{locale:?}: {text:?} is cut off or missing:\n{screen}"
                );
            }
        }
    }
}
