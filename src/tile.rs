use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};
use serde::{Deserialize, Serialize};
use uid::Id;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Tile {
    Empty(#[serde(skip)] Id<Tile>),
    Blocker(#[serde(skip)] Id<Tile>),
    Regular(#[serde(skip)] Id<Tile>, Color), // color
}

impl Widget for &Tile {
    fn render(self, rect: Rect, buf: &mut Buffer) {
        let rect = Rect::new(rect.x, rect.y, rect.width - 2, rect.height - 1);
        match self {
            Tile::Empty(_) => {
                buf.set_style(rect, Style::default().bg(Color::Black));
            }
            Tile::Blocker(_) => {
                for x in rect.left()..rect.right() {
                    for y in rect.top()..rect.bottom() {
                        let symbol = if (x + y) % 2 == 0 { "░" } else { "▒" };
                        buf[(x, y)].set_symbol(symbol).set_fg(Color::Gray);
                    }
                }
            }
            Tile::Regular(_, color_code) => {
                buf.set_style(rect, Style::default().bg(*color_code).fg(Color::White));
            }
        }
    }
}
