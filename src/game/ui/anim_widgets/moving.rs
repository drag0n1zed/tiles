use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::{Color, Style},
    widgets::Widget,
};

use crate::game::logic::grid::{MoveDir, tile::Tile};

pub struct MovingTile<'a> {
    tile: &'a Tile,
    from: Rect,
    to: Rect,
    dir: MoveDir,
    t: f64,
}

impl<'a> MovingTile<'a> {
    pub fn new(tile: &'a Tile, from: Rect, to: Rect, dir: MoveDir, t: f64) -> Self {
        Self { tile, from, to, dir, t }
    }
}

impl<'a> Widget for MovingTile<'a> {
    fn render(self, _rect: Rect, buf: &mut Buffer) {
        let Tile::Regular { color, .. } = self.tile else {
            return;
        };

        match self.dir {
            MoveDir::Up | MoveDir::Down => self.render_vertical(*color, buf),
            MoveDir::Left | MoveDir::Right => self.render_horizontal(*color, buf),
        }
    }
}

impl<'a> MovingTile<'a> {
    fn render_vertical(self, color: Color, buf: &mut Buffer) {
        let start_y = self.from.y as f64;
        let end_y = self.to.y as f64;
        let current_y = start_y + (end_y - start_y) * self.t;

        let y_int = current_y.floor() as u16;
        let offset = current_y.fract().clamp(0.0, 1.0);

        let rect = Rect::new(self.from.x, y_int, self.from.width, self.from.height);

        let d = (offset * 2.0).round() as usize;
        let top_symbols = ["█", "▄", " "];
        let bottom_symbols = ["", "▀", "█"];

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

    fn render_horizontal(self, color: Color, buf: &mut Buffer) {
        let start_x = self.from.x as f64;
        let end_x = self.to.x as f64;
        let current_x = start_x + (end_x - start_x) * self.t;

        let x_int = current_x.floor() as u16;
        let offset = current_x.fract().clamp(0.0, 1.0);

        let rect = Rect::new(x_int, self.from.y, self.from.width, self.from.height);

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

        if rect.width > 1 {
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
