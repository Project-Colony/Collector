//! The styles every drawing function picks from, by role rather than by
//! colour. The colours are the active theme's (src/theme.rs).

use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::Block;

use crate::theme;

/// The whole screen: the theme's background and main text colour. Every
/// other style is drawn over it.
pub fn base() -> Style {
    let palette = theme::active();
    Style::new().fg(palette.text_primary).bg(palette.bg_primary)
}

/// Page and category headings, section and box titles.
pub fn title() -> Style {
    Style::new()
        .fg(theme::active().text_primary)
        .add_modifier(Modifier::BOLD)
}

/// A setting's value, and anything one step below the main text.
pub fn secondary() -> Style {
    Style::new().fg(theme::active().text_secondary)
}

/// Descriptions: read once, then out of the way.
pub fn muted() -> Style {
    Style::new().fg(theme::active().text_muted)
}

/// The footer's key hints, a step further down.
pub fn hint() -> Style {
    Style::new().fg(theme::active().text_dim)
}

/// Mount points: the accent, without a background.
pub fn accent() -> Style {
    Style::new().fg(theme::active().accent)
}

/// A gauge: filled in the accent over the progress track.
pub fn gauge() -> Style {
    let palette = theme::active();
    Style::new().fg(palette.accent).bg(palette.bg_progress)
}

/// Something that can be clicked or focused. Selected sits on the accent, as
/// colony-ui's `selection_style` draws it; hovered gets the hover background,
/// distinct from selected.
pub fn item(selected: bool, hovered: bool) -> Style {
    let palette = theme::active();
    if selected {
        Style::new().fg(palette.on_accent).bg(palette.accent)
    } else if hovered {
        Style::new().bg(palette.bg_card_hover)
    } else {
        Style::new()
    }
}

/// The identity button: the program's name in the accent, on the selected
/// background while Preferences is open, as colony-ui's `identity_button`.
pub fn identity(open: bool, hovered: bool) -> Style {
    let palette = theme::active();
    let style = Style::new().fg(palette.accent).add_modifier(Modifier::BOLD);
    if open {
        style.bg(palette.bg_selected)
    } else if hovered {
        style.bg(palette.bg_card_hover)
    } else {
        style
    }
}

/// The line between the category list and the page's content.
pub fn divider() -> Style {
    Style::new().fg(theme::active().divider)
}

/// The toast that confirms a change, as colony-ui draws it: `success` on
/// `success_bg`.
pub fn toast() -> Style {
    let palette = theme::active();
    Style::new().fg(palette.success).bg(palette.success_bg)
}

/// A problem with the preferences file.
pub fn warning() -> Style {
    Style::new().fg(theme::active().warning)
}

/// A monitor box: a subtle border around the content, its title on top.
pub fn block(title: &str) -> Block<'_> {
    Block::bordered()
        .border_style(Style::new().fg(theme::active().border_subtle))
        .title(Span::styled(title, self::title()))
}
