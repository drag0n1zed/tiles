use std::error::Error;

use ndarray::prelude::*;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use uid::Id;
use union_find::{QuickUnionUf, UnionBySize, UnionFind};

use crate::tile::Tile;

#[derive(Clone, Copy)]
pub enum TileMoveDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(into = "VecGrid", from = "VecGrid")]
pub struct Grid {
    array: Array2<Tile>,
}

#[derive(Serialize, Deserialize)]
struct VecGrid {
    height: usize,
    width: usize,
    pattern: Vec<Vec<Tile>>,
}

impl From<Grid> for VecGrid {
    fn from(grid: Grid) -> Self {
        let pattern = grid
            .get_array()
            .rows()
            .into_iter()
            .map(|chunk| chunk.to_vec())
            .collect();

        VecGrid {
            width: grid.get_width(),
            height: grid.get_height(),
            pattern,
        }
    }
}

impl From<VecGrid> for Grid {
    fn from(vec_grid: VecGrid) -> Self {
        let vec_flat: Vec<Tile> = vec_grid.pattern.into_iter().flatten().collect();
        let data = Array2::from_shape_vec((vec_grid.height, vec_grid.width), vec_flat).unwrap();
        Grid { array: data }
    }
}

impl Grid {
    pub fn new(length: usize, width: usize) -> Self {
        Self {
            array: Array2::from_elem((length, width), Tile::Empty(Id::new())),
        }
    }

    pub fn to_ron(&self) -> String {
        let pretty_config = PrettyConfig::new().depth_limit(2);
        ron::ser::to_string_pretty(&self, pretty_config).unwrap()
    }

    pub fn from_ron(ron: &str) -> Result<Self, Box<dyn Error>> {
        Ok(ron::de::from_str(ron)?)
    }

    pub fn get_array(&self) -> &Array2<Tile> {
        &self.array
    }

    pub fn get_height(&self) -> usize {
        self.array.dim().0
    }

    pub fn get_width(&self) -> usize {
        self.array.dim().1
    }

    pub fn move_grid(&mut self, direction: TileMoveDirection) {
        match direction {
            TileMoveDirection::Left => {
                for x in 0..self.get_width() {
                    for y in 0..self.get_height() {
                        self.move_tile(y, x, direction);
                    }
                }
            }
            TileMoveDirection::Right => {
                for x in (0..self.get_width()).rev() {
                    for y in 0..self.get_height() {
                        self.move_tile(y, x, direction);
                    }
                }
            }
            TileMoveDirection::Up => {
                for y in 0..self.get_height() {
                    for x in 0..self.get_width() {
                        self.move_tile(y, x, direction);
                    }
                }
            }
            TileMoveDirection::Down => {
                for y in (0..self.get_height()).rev() {
                    for x in 0..self.get_width() {
                        self.move_tile(y, x, direction);
                    }
                }
            }
        };

        self.clear_connected_tiles();
    }

    fn move_tile(&mut self, y: usize, x: usize, direction: TileMoveDirection) {
        let (ty, tx) = match direction {
            TileMoveDirection::Left => (y, x.wrapping_sub(1)), // (y, x - 1)
            TileMoveDirection::Right => (y, x.wrapping_add(1)), // (y, x + 1)
            TileMoveDirection::Up => (y.wrapping_sub(1), x),   // (y - 1, x)
            TileMoveDirection::Down => (y.wrapping_add(1), x), // (y + 1, x)
        };

        let from = self.array.get((y, x)).unwrap();
        let Some(to) = self.array.get((ty, tx)) else {
            // Hit the wall
            return;
        };

        // Target is regular or blocker tile: cannot move
        // Origin is empty or blocker tile: cannot move
        if let (Tile::Regular(_, _), Tile::Empty(_)) = (from, to) {
            self.array.swap((y, x), (ty, tx));
        }
    }

    fn clear_connected_tiles(&mut self) {
        let (length, width) = (self.get_height(), self.get_width());
        let mut uf = QuickUnionUf::<UnionBySize>::new(length * width);
        for ((y, x), tile) in self.array.indexed_iter() {
            let Tile::Regular(_, color) = tile else {
                continue;
            };
            let index = y * width + x;

            if x + 1 < width {
                if let Tile::Regular(_, right_color) = self.array.get((y, x + 1)).unwrap() {
                    if color == right_color {
                        uf.union(index, index + 1);
                    }
                }
            }
            if y + 1 < length {
                if let Tile::Regular(_, bottom_color) = self.array.get((y + 1, x)).unwrap() {
                    if color == bottom_color {
                        uf.union(index, index + width);
                    }
                }
            }
        }

        for ((y, x), tile) in self.array.indexed_iter_mut() {
            let index = y * width + x;
            let root_index = uf.find(index);
            if uf.get(root_index).size() >= 4 {
                *tile = Tile::Empty(Id::new());
            }
        }
    }
}

impl Widget for &Grid {
    fn render(self, rect: Rect, buf: &mut Buffer) {
        // Aspect ratio adjustment
        let grid_rect = {
            let (rect_w, rect_h) = (rect.width, rect.height);
            let (grid_w, grid_h) = (self.get_width() as u16, self.get_height() as u16);
            let ratio = (rect_w / (grid_w * 2)).min(rect_h / grid_h).max(1);
            let (new_rect_w, new_rect_h) = (grid_w * ratio * 2, grid_h * ratio);
            let (start_x, start_y) = (
                rect.x + (rect_w.saturating_sub(new_rect_w)) / 2,
                rect.y + (rect_h.saturating_sub(new_rect_h)) / 2,
            );
            Rect::new(start_x, start_y, new_rect_w, new_rect_h)
        };

        let row_constraints =
            (0..self.get_height()).map(|_| Constraint::Ratio(1, self.get_height() as u32));

        let row_rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(grid_rect);

        for (y, row_rect) in row_rects.iter().enumerate() {
            let col_constraints =
                (0..self.get_width()).map(|_| Constraint::Ratio(1, self.get_width() as u32));

            let col_rects = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(col_constraints)
                .split(*row_rect);

            for (x, cell_rect) in col_rects.iter().enumerate() {
                self.get_array()
                    .get((y, x))
                    .unwrap()
                    .render(*cell_rect, buf);
            }
        }
    }
}
