//! Drawing: one file per box.

mod cpu_memory;
mod disks;
mod network;
mod processes;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};

use crate::app::App;

pub fn render_ui(frame: &mut Frame, app: &App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(7),
            Constraint::Min(10),
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
}
