use ratatui::{
    style::{Color, Style},
    text::Span,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub enum Tile {
    #[default]
    Empty,
    Blocker,
    Regular(u8), // color
}

impl From<&Tile> for Span<'static> {
    fn from(tile: &Tile) -> Self {
        match tile {
            Tile::Empty => Span::styled("[ ]", Style::default().fg(Color::White)),
            Tile::Blocker => Span::styled("[#]", Style::default().fg(Color::Black).bold()),
            Tile::Regular(color) => {
                Span::styled("[x]", Style::default().fg(Color::Indexed(*color)))
            }
        }
    }
}
