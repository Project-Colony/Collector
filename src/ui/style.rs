//! The styles every drawing function picks from, by role rather than by
//! colour.

use ratatui::style::{Modifier, Style};

/// Page and category headings, section titles.
pub fn title() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}

/// Descriptions and hints: read once, then out of the way.
pub fn muted() -> Style {
    Style::default().add_modifier(Modifier::DIM)
}

/// Something that can be clicked or focused: selected stands out by its
/// background, hovered by a mark of its own, distinct from selected.
pub fn item(selected: bool, hovered: bool) -> Style {
    if selected {
        Style::default().add_modifier(Modifier::REVERSED)
    } else if hovered {
        Style::default().add_modifier(Modifier::UNDERLINED)
    } else {
        Style::default()
    }
}
