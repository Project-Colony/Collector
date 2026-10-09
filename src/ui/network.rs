use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let (rx, tx) = app.network_rate;
    let net_lines = vec![
        Line::from(format!("RX: {:.1} KB/s", rx / 1024.0)),
        Line::from(format!("TX: {:.1} KB/s", tx / 1024.0)),
        Line::from("q or Ctrl+C to quit"),
    ];
    let net_block =
        Paragraph::new(net_lines).block(Block::default().title("Network").borders(Borders::ALL));
    frame.render_widget(net_block, area);
}
