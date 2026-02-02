pub mod anim;
pub mod tile;
mod vec_grid;

use std::time::Instant;

use color_eyre::eyre::{Ok, Result};
use ndarray::prelude::*;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use union_find::{QuickUnionUf, UnionBySize, UnionFind};

use anim::Animation;
use tile::Tile;
use vec_grid::VecGrid;

#[derive(Serialize, Deserialize, Clone)]
#[serde(into = "VecGrid", from = "VecGrid")]
pub struct Grid {
    pub steps: usize,
    pub tiles: Array2<Tile>,
    #[serde(skip)]
    pub active_animations: Vec<Animation>,
    #[serde(skip)]
    pub pending_pop: bool,
}

#[derive(Clone, Copy)]
pub enum MoveDir {
    Up,
    Down,
    Left,
    Right,
}

impl Grid {
    pub fn new(length: usize, width: usize, steps: usize) -> Self {
        Self {
            tiles: Array2::from_elem((length, width), Tile::Empty),
            steps,
            active_animations: Vec::new(),
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
        self.tiles.dim().0
    }

    pub fn get_width(&self) -> usize {
        self.tiles.dim().1
    }

    pub fn get_tiles_view(&self) -> ArrayView2<'_, Tile> {
        self.tiles.view()
    }

    pub fn get_anims_slice(&self) -> &[Animation] {
        &self.active_animations
    }

    pub fn is_anim_completed(&self) -> bool {
        self.active_animations.is_empty()
    }

    // Returns true if move happened, false if no move happened
    pub fn move_grid(&mut self, direction: MoveDir) -> bool {
        if self.steps == 0 {
            return false;
        }
        let mut moved = false;
        match direction {
            MoveDir::Left => {
                for dx in 0..self.get_width() {
                    for dy in 0..self.get_height() {
                        moved |= self.move_tile(dy, dx, direction);
                    }
                }
            }
            MoveDir::Right => {
                for dx in (0..self.get_width()).rev() {
                    for dy in 0..self.get_height() {
                        moved |= self.move_tile(dy, dx, direction);
                    }
                }
            }
            MoveDir::Up => {
                for dy in 0..self.get_height() {
                    for dx in 0..self.get_width() {
                        moved |= self.move_tile(dy, dx, direction);
                    }
                }
            }
            MoveDir::Down => {
                for dy in (0..self.get_height()).rev() {
                    for dx in 0..self.get_width() {
                        moved |= self.move_tile(dy, dx, direction);
                    }
                }
            }
        };
        if moved {
            self.steps = self.steps.saturating_sub(1);
            true
        } else {
            false
        }
    }

    // Returns true if tile moved, false if tile did not move
    fn move_tile(&mut self, y: usize, x: usize, direction: MoveDir) -> bool {
        let (ty, tx) = match direction {
            MoveDir::Left => (y, x.wrapping_sub(1)),  // (y, x - 1)
            MoveDir::Right => (y, x.wrapping_add(1)), // (y, x + 1)
            MoveDir::Up => (y.wrapping_sub(1), x),    // (y - 1, x)
            MoveDir::Down => (y.wrapping_add(1), x),  // (y + 1, x)
        };

        let from = self.tiles.get((y, x)).unwrap();
        let Some(to) = self.tiles.get((ty, tx)) else {
            // Hit the wall
            return false;
        };

        // Target is regular or blocker tile: cannot move
        // Origin is empty or blocker tile: cannot move
        if let (Tile::Regular { .. }, Tile::Empty) = (from, to) {
            self.active_animations.push(Animation::Moving {
                tile: *from,
                from: (y, x),
                direction,
                start_time: Instant::now(),
            });
            self.tiles.swap((y, x), (ty, tx));
            self.pending_pop = true;
            true
        } else {
            false
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
    }

    fn pop_connected_tiles(&mut self) {
        let (length, width) = (self.get_height(), self.get_width());
        let mut uf = QuickUnionUf::<UnionBySize>::new(length * width);
        for ((y, x), tile) in self.tiles.indexed_iter() {
            let Tile::Regular { color, .. } = tile else {
                continue;
            };
            let index = y * width + x;

            let neighbors = [(x + 1, y, index + 1), (x, y + 1, index + width)]; // Right, Down
            for (nx, ny, n_index) in neighbors {
                if matches!(&self.tiles.get((ny, nx)), Some(Tile::Regular { color: c, .. }) if c == color) {
                    uf.union(index, n_index);
                }
            }
        }

        for ((y, x), tile) in self.tiles.indexed_iter_mut() {
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

                *tile = Tile::Empty;
            }
        }
    }
}
