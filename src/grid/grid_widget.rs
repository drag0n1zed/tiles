use super::{Animation, Tile, grid_layout::GridLayout};
use ndarray::ArrayView2;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
};

pub struct GridWidget<'a> {
    pub tiles: ArrayView2<'a, Tile>,
    pub animations: &'a [Animation],
    pub animation_mask: ArrayView2<'a, bool>,
}

impl<'a> Widget for GridWidget<'a> {
    fn render(self, rect: Rect, buf: &mut Buffer) {
        let (height, width) = self.tiles.dim();

        let grid_rect = {
            let (rect_w, rect_h) = (rect.width, rect.height);
            let (grid_w, grid_h) = (width as u16, height as u16);

            let ratio = (rect_w / (grid_w * 2)).min(rect_h / grid_h).max(1);
            let (new_rect_w, new_rect_h) = (grid_w * ratio * 2, grid_h * ratio);

            let (start_x, start_y) = (
                rect.x + (rect_w.saturating_sub(new_rect_w)) / 2,
                rect.y + (rect_h.saturating_sub(new_rect_h)) / 2,
            );
            Rect::new(start_x, start_y, new_rect_w, new_rect_h)
        };

        let row_constraints = vec![Constraint::Ratio(1, height as u32); height];
        let col_constraints = vec![Constraint::Ratio(1, width as u32); width];

        let row_rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(grid_rect);

        let mut rect_lookup = Vec::with_capacity(height * width);

        for (y, row_rect) in row_rects.iter().enumerate() {
            let col_rects = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(col_constraints.clone())
                .split(*row_rect);

            for (x, tile_rect) in col_rects.iter().enumerate() {
                rect_lookup.push(*tile_rect);

                if self.animation_mask[[y, x]] {
                    Tile::Empty.render(*tile_rect, buf);
                } else {
                    self.tiles[[y, x]].render(*tile_rect, buf);
                }
            }
        }

        let layout = &GridLayout::new(rect_lookup, width);

        for animation in self.animations {
            animation
                .with_layout(layout)
                .render(Default::default(), buf);
        }
    }
}
