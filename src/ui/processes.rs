use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Row, Table};

use crate::app::App;

const PROCESS_LIMIT: usize = 10;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let mut processes: Vec<_> = app.system.processes().values().collect();
    let cpu_count = app.system.cpus().len().max(1) as f32;
    processes.sort_by(|a, b| {
        let a_usage = a.cpu_usage() / cpu_count;
        let b_usage = b.cpu_usage() / cpu_count;
        b_usage.total_cmp(&a_usage)
    });

    let rows = processes.into_iter().take(PROCESS_LIMIT).map(|process| {
        let cpu_usage = process.cpu_usage() / cpu_count;
        Row::new(vec![
            process.name().to_string_lossy().into_owned(),
            format!("{:.1}%", cpu_usage),
            format!("{:.1} MB", process.memory() as f64 / 1_048_576.0),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(50),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ],
    )
    .header(Row::new(vec!["Process", "CPU", "Memory"]).style(Style::default().fg(Color::Green)))
    .block(
        Block::default()
            .title("Top Processes")
            .borders(Borders::ALL),
    );

    frame.render_widget(table, area);
}
