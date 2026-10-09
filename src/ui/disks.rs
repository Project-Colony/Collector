use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    // By mount point: btrfs subvolumes and bind mounts repeat one device name,
    // so the device alone could not tell the rows apart.
    let disk_lines: Vec<Line> = app
        .mounted_disks()
        .into_iter()
        .map(|(mount_point, disk)| {
            let total = disk.total_space() as f64;
            let available = disk.available_space() as f64;
            let used = total - available;
            Line::from(vec![
                Span::styled(
                    format!("{} ", mount_point.display()),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(format!(
                    "{:.1}/{:.1} GB",
                    used / 1_073_741_824.0,
                    total / 1_073_741_824.0
                )),
            ])
        })
        .collect();

    let disk_block =
        Paragraph::new(disk_lines).block(Block::default().title("Disks").borders(Borders::ALL));
    frame.render_widget(disk_block, area);
}
