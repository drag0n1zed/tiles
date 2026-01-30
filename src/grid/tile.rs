use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::{Color, Style},
    widgets::Widget,
};
use serde::{Deserialize, Serialize};
use uid::Id;

use super::MoveDirection;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Tile {
    Empty,
    Blocker,
    Regular {
        #[serde(skip)]
        id: Id<Tile>,
        color: Color,
    },
}

impl Widget for &Tile {
    fn render(self, rect: Rect, buf: &mut Buffer) {
        let rect = Rect::new(
            rect.x,
            rect.y,
            rect.width.saturating_sub(2),
            rect.height.saturating_sub(1),
        );
        match self {
            Tile::Empty => {
                buf.set_style(rect, Style::default().bg(Color::Black));
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

impl Tile {
    pub fn with_offset(&self, direction: MoveDirection, offset: f64) -> OffsetTile<'_> {
        OffsetTile {
            tile: self,
            direction,
            offset,
        }
    }
}

pub struct OffsetTile<'a> {
    tile: &'a Tile,
    direction: MoveDirection,
    offset: f64,
}

impl<'a> Widget for OffsetTile<'a> {
    fn render(self, rect: Rect, buf: &mut Buffer) {
        let Tile::Regular { color, .. } = self.tile else {
            return;
        };
        let rect = Rect::new(
            rect.x,
            rect.y,
            rect.width.saturating_sub(2),
            rect.height.saturating_sub(1),
        );

        match self.direction {
            MoveDirection::Up | MoveDirection::Down => self.render_vertical(rect, *color, buf),
            MoveDirection::Left | MoveDirection::Right => self.render_horizontal(rect, *color, buf),
        }
    }
}

impl<'a> OffsetTile<'a> {
    fn render_vertical(self, rect: Rect, color: Color, buf: &mut Buffer) {
        let offset = self.offset.clamp(0.0, 1.0);

        let d = (offset * 2.0).round() as usize;

        let top_symbols = ["█", "▄", " "];
        let bottom_symbols = ["", "▀", "█"];

        // Requires Unicode 13.0, unused because of weird artifacts in fonts
        // let symbols = ["", "█", "🮆", "🮅", "🮄", "▀", "🮃", "🮂", "▔"];

        let top_symbol = top_symbols[d];
        let bottom_symbol = bottom_symbols[d];

        if !top_symbol.is_empty() {
            for dx in 0..rect.width {
                if let Some(cell) = buf.cell_mut(Position::new(rect.x + dx, rect.y)) {
                    cell.set_symbol(top_symbol).set_fg(color);
                }
            }
        }

        if rect.height > 1 {
            let body_rect = Rect::new(rect.x, rect.y + 1, rect.width, rect.height - 1);
            buf.set_style(body_rect, Style::default().bg(color));
        }

        if !bottom_symbol.is_empty() {
            let overflow_y = rect.y + rect.height;
            for dx in 0..rect.width {
                if let Some(cell) = buf.cell_mut(Position::new(rect.x + dx, overflow_y)) {
                    cell.set_symbol(bottom_symbol).set_fg(color);
                }
            }
        }
    }
    fn render_horizontal(self, rect: Rect, color: Color, buf: &mut Buffer) {
        let offset = self.offset.clamp(0.0, 1.0);

        let d = (offset * 2.0).round() as usize;

        let left_symbols = ["█", "▐", ""];
        let right_symbols = ["", "▌", "█"];

        let left_symbol = left_symbols[d];
        let right_symbol = right_symbols[d];

        if !left_symbol.is_empty() {
            for dy in 0..rect.height {
                if let Some(cell) = buf.cell_mut(Position::new(rect.x, rect.y + dy)) {
                    cell.set_symbol(left_symbol).set_fg(color);
                }
            }
        }

        if rect.height > 1 {
            let body_rect = Rect::new(rect.x + 1, rect.y, rect.width - 1, rect.height);
            buf.set_style(body_rect, Style::default().bg(color));
        }

        if !right_symbol.is_empty() {
            let overflow_x = rect.x + rect.width;
            for dy in 0..rect.height {
                if let Some(cell) = buf.cell_mut(Position::new(overflow_x, rect.y + dy)) {
                    cell.set_symbol(right_symbol).set_fg(color);
                }
            }
        }
    }
}
