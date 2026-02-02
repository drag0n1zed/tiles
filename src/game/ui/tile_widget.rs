use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::game::logic::grid::tile::Tile;

impl Widget for &Tile {
    fn render(self, rect: Rect, buf: &mut Buffer) {
        match self {
            Tile::Empty => {
                buf.set_style(rect, Style::default().bg(Color::DarkGray));
            }
            Tile::Blocker => {
                /*
                for x in rect.left()..rect.right() {
                    for y in rect.top()..rect.bottom() {
                        let symbol = if (x + y) % 2 == 0 { "░" } else { "▒" };
                        buf[(x, y)].set_symbol(symbol).set_fg(Color::Gray);
                    }
                }
                */
            }
            Tile::Regular { color, .. } => {
                buf.set_style(rect, Style::default().bg(*color));
            }
        }
    }
}
