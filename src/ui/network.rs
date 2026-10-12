use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::i18n::{number, t, tf};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let (rx, tx) = app.network_rate;
    let net_lines = vec![
        Line::from(tf("rx_rate", &[("rate", &number(rx / 1024.0, 1))])),
        Line::from(tf("tx_rate", &[("rate", &number(tx / 1024.0, 1))])),
    ];
    let net_block =
        Paragraph::new(net_lines).block(Block::default().title(t("network")).borders(Borders::ALL));
    frame.render_widget(net_block, area);
}
