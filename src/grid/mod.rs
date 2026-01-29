mod animation;
mod tile;
mod vec_grid;
mod widget;

use std::time::Instant;

use color_eyre::eyre::{Ok, Result};
use ndarray::prelude::*;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use union_find::{QuickUnionUf, UnionBySize, UnionFind};

use animation::Animation;
use tile::Tile;
use vec_grid::VecGrid;

use crate::grid::widget::GridWidget;

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

    pub fn as_widget(&self) -> GridWidget<'_> {
        GridWidget {
            tiles: self.data.view(),
            animations: &self.active_animations,
            animation_mask: self.animation_mask.view(),
        }
    }

    pub fn get_height(&self) -> usize {
        self.data.dim().0
    }

    pub fn get_width(&self) -> usize {
        self.data.dim().1
    }

    pub fn is_anim_completed(&self) -> bool {
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
}
