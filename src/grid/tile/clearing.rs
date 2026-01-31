use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    widgets::Widget,
};

use crate::grid::tile::Tile;

impl Tile {
    pub fn with_clearing_progress(&self, at: Rect, t: f64) -> ClearingTile<'_> {
        ClearingTile { tile: self, at, t }
    }
}

pub struct ClearingTile<'a> {
    tile: &'a Tile,
    at: Rect,
    t: f64,
}

impl<'a> Widget for ClearingTile<'a> {
    fn render(self, _rect: Rect, buf: &mut Buffer) {
        let Tile::Regular { color, .. } = self.tile else {
            return;
        };
        let area = self.at;
        if area.width == 0 || area.height == 0 {
            return;
        }
        let center_x = area.x as f64 + (area.width as f64 / 2.0);
        let center_y = (area.y as f64 * 2.0) + (area.height as f64);

        let corner_dx = area.x as f64 - center_x;
        let corner_dy = (area.y as f64 * 2.0) - center_y;
        let max_dist = (corner_dx.powi(2) + corner_dy.powi(2)).sqrt();

        let current_radius = max_dist * self.t;

        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let check_vis = |off_x: f64, off_y: f64| -> bool {
                    let px = x as f64 + off_x;
                    let py = (y as f64 * 2.0) + (off_y * 2.0);
                    let dx = px - center_x;
                    let dy = py - center_y;
                    (dx.powi(2) + dy.powi(2)).sqrt() <= current_radius
                };

                let tl = check_vis(0.25, 0.25);
                let tr = check_vis(0.75, 0.25);
                let bl = check_vis(0.25, 0.75);
                let br = check_vis(0.75, 0.75);

                let symbol = match (tl, tr, bl, br) {
                    (true, true, true, true) => "█",
                    (true, true, true, false) => "▛",
                    (true, true, false, true) => "▜",
                    (true, false, true, true) => "▙",
                    (false, true, true, true) => "▟",
                    (true, true, false, false) => "▀",
                    (false, false, true, true) => "▄",
                    (true, false, true, false) => "▌",
                    (false, true, false, true) => "▐",
                    (true, false, false, true) => "▚",
                    (false, true, true, false) => "▞",
                    (true, false, false, false) => "▘",
                    (false, true, false, false) => "▝",
                    (false, false, true, false) => "▖",
                    (false, false, false, true) => "▗",
                    (false, false, false, false) => " ",
                };

                if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                    cell.set_symbol(symbol).set_fg(*color);
                }
            }
        }
    }
}
