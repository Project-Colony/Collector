use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Gauge};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let cpu_usage = app.system.global_cpu_usage();
    let cpu_gauge = Gauge::default()
        .block(Block::default().title("CPU").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(gauge_percent(f64::from(cpu_usage)));
    frame.render_widget(cpu_gauge, layout[0]);

    let total_memory = app.system.total_memory() as f64;
    let used_memory = app.system.used_memory() as f64;
    let memory_percent = if total_memory > 0.0 {
        (used_memory / total_memory) * 100.0
    } else {
        0.0
    };

    let memory_gauge = Gauge::default()
        .block(Block::default().title("Memory").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Magenta))
        .percent(gauge_percent(memory_percent))
        .label(format!(
            "{:.1} / {:.1} GB",
            used_memory / 1_073_741_824.0,
            total_memory / 1_073_741_824.0
        ));
    frame.render_widget(memory_gauge, layout[1]);
}

/// A reading as a gauge percentage. `Gauge::percent` panics above 100, and a
/// reading can go over: used memory can exceed the total under cgroup
/// accounting, and a cast alone would turn a huge value into 65535.
fn gauge_percent(value: f64) -> u16 {
    if value.is_finite() {
        value.round().clamp(0.0, 100.0) as u16
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gauge_percent_stays_in_range() {
        assert_eq!(gauge_percent(f64::NAN), 0);
        assert_eq!(gauge_percent(f64::INFINITY), 0);
        assert_eq!(gauge_percent(-3.0), 0);
        assert_eq!(gauge_percent(150.0), 100);
        assert_eq!(gauge_percent(42.4), 42);
    }
}
