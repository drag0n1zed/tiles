mod animation;
mod tile;
mod vec_grid;

use std::time::Instant;

use color_eyre::eyre::{Ok, Result};
use ndarray::prelude::*;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use union_find::{QuickUnionUf, UnionBySize, UnionFind};

use animation::Animation;
use tile::Tile;
use vec_grid::VecGrid;

#[derive(Serialize, Deserialize, Clone)]
#[serde(into = "VecGrid", from = "VecGrid")]
pub struct Grid {
    pub steps: usize,
    pub data: Array2<Tile>,
    #[serde(skip)]
    pub active_animations: Vec<Animation>,
    #[serde(skip)]
    animation_mask: Array2<bool>,
    #[serde(skip)]
    pending_pop: bool,
}

#[derive(Clone, Copy)]
pub enum TileMoveDirection {
    Up,
    Down,
    Left,
    Right,
}

impl Grid {
    pub fn new(length: usize, width: usize, steps: usize) -> Self {
        Self {
            data: Array2::from_elem((length, width), Tile::Empty),
            steps,
            active_animations: Vec::new(),
            animation_mask: Array2::from_elem((length, width), false),
            pending_pop: false,
        }
    }

    pub fn to_ron(&self) -> String {
        let pretty_config = PrettyConfig::new().depth_limit(2);
        ron::ser::to_string_pretty(&self, pretty_config).unwrap()
    }

    pub fn from_ron(ron: &str) -> Result<Self> {
        Ok(ron::de::from_str(ron)?)
    }

    pub fn get_height(&self) -> usize {
        self.data.dim().0
    }

    pub fn get_width(&self) -> usize {
        self.data.dim().1
    }

    pub fn anim_completed(&self) -> bool {
        self.active_animations.is_empty()
    }

    pub fn move_grid(&mut self, direction: TileMoveDirection) {
        if self.steps == 0 {
            return;
        }
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

        self.steps = self.steps.saturating_sub(1);
    }

    fn move_tile(&mut self, y: usize, x: usize, direction: TileMoveDirection) {
        let (ty, tx) = match direction {
            TileMoveDirection::Left => (y, x.wrapping_sub(1)), // (y, x - 1)
            TileMoveDirection::Right => (y, x.wrapping_add(1)), // (y, x + 1)
            TileMoveDirection::Up => (y.wrapping_sub(1), x),   // (y - 1, x)
            TileMoveDirection::Down => (y.wrapping_add(1), x), // (y + 1, x)
        };

        let from = self.data.get((y, x)).unwrap();
        let Some(to) = self.data.get((ty, tx)) else {
            // Hit the wall
            return;
        };

        // Target is regular or blocker tile: cannot move
        // Origin is empty or blocker tile: cannot move
        if let (Tile::Regular { .. }, Tile::Empty) = (from, to) {
            self.active_animations.push(Animation::Moving {
                tile: *from,
                from: (y, x),
                to: (ty, tx),
                start_time: Instant::now(),
            });
            self.animation_mask[[y, x]] = true;
            self.animation_mask[[ty, tx]] = true;
            self.data.swap((y, x), (ty, tx));
            self.pending_pop = true;
        }
    }

    fn pop_connected_tiles(&mut self) {
        let (length, width) = (self.get_height(), self.get_width());
        let mut uf = QuickUnionUf::<UnionBySize>::new(length * width);
        for ((y, x), tile) in self.data.indexed_iter() {
            let Tile::Regular { color, .. } = tile else {
                continue;
            };
            let index = y * width + x;

            let neighbors = [(x + 1, y, index + 1), (x, y + 1, index + width)]; // Right, Down
            for (nx, ny, n_index) in neighbors {
                if matches!(&self.data.get((ny, nx)), Some(Tile::Regular { color: c, .. }) if c == color)
                {
                    uf.union(index, n_index);
                }
            }
        }

        for ((y, x), tile) in self.data.indexed_iter_mut() {
            let Tile::Regular { .. } = tile else {
                continue;
            };

            let index = y * width + x;
            let root_index = uf.find(index);
            if uf.get(root_index).size() >= 4 {
                self.active_animations.push(Animation::Clearing {
                    tile: *tile,
                    at: (y, x),
                    start_time: Instant::now(),
                });
                self.animation_mask[[y, x]] = true;

                *tile = Tile::Empty;
            }
        }
    }

    pub fn update_anim_state(&mut self) {
        self.clear_completed_animations();

        if self.active_animations.is_empty() && self.pending_pop {
            self.pending_pop = false;
            self.pop_connected_tiles(); // Populates animations again
        }
    }

    fn clear_completed_animations(&mut self) {
        self.active_animations.retain(|anim| anim.is_active());

        self.animation_mask.fill(false);

        for anim in &self.active_animations {
            match anim {
                Animation::Moving { from, to, .. } => {
                    self.animation_mask[*from] = true;
                    self.animation_mask[*to] = true;
                }
                Animation::Clearing { at, .. } => {
                    self.animation_mask[*at] = true;
                }
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

        let mut rect_lookup_table = Array2::from_elem((self.get_height(), self.get_width()), None);

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

            for (x, tile_rect) in col_rects.iter().enumerate() {
                if self.animation_mask[[y, x]] {
                    let empty = Tile::Empty;
                    empty.render(*tile_rect, buf);
                    rect_lookup_table[[y, x]] = Some(*tile_rect);
                } else {
                    self.data[[y, x]].render(*tile_rect, buf);
                }
            }
        }

        for animation in &self.active_animations {
            match animation {
                Animation::Moving { from, to, .. } => {
                    animation.render_moving(
                        rect_lookup_table[*from].unwrap(),
                        rect_lookup_table[*to].unwrap(),
                        buf,
                    );
                }
                Animation::Clearing { at, .. } => {
                    animation.render_clearing(rect_lookup_table[*at].unwrap(), buf);
                }
            }
        }
    }
}
