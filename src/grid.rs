use std::error::Error;

use ndarray::prelude::*;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use union_find::{QuickUnionUf, UnionBySize, UnionFind};

use crate::tile::Tile;

#[derive(Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(into = "VecGrid", from = "VecGrid")]
pub struct Grid {
    data: Array2<Tile>,
}

#[derive(Serialize, Deserialize)]
struct VecGrid {
    length: usize,
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
            length: grid.get_length(),
            pattern,
        }
    }
}

impl From<VecGrid> for Grid {
    fn from(vec_grid: VecGrid) -> Self {
        let vec_flat: Vec<Tile> = vec_grid.pattern.into_iter().flatten().collect();
        let data =
            Array2::from_shape_vec((vec_grid.length, vec_grid.width), vec_flat).unwrap_or_default();
        Grid { data }
    }
}

impl Grid {
    pub fn new(length: usize, width: usize) -> Self {
        Self {
            data: Array2::from_elem((length, width), Tile::Empty),
        }
    }

    pub fn get_array(&self) -> &Array2<Tile> {
        &self.data
    }

    pub fn get_length(&self) -> usize {
        self.data.dim().0
    }

    pub fn get_width(&self) -> usize {
        self.data.dim().1
    }

    pub fn get(&self, y: usize, x: usize) -> Option<&Tile> {
        self.data.get((y, x))
    }

    pub fn set(&mut self, y: usize, x: usize, tile: Tile) -> bool {
        match self.data.get_mut((y, x)) {
            Some(val) => {
                *val = tile;
                true
            }
            None => false,
        }
    }

    pub fn move_grid(&mut self, direction: Direction) {
        match direction {
            Direction::Left => {
                for x in 0..self.get_width() {
                    for y in 0..self.get_length() {
                        self.move_tile(y, x, direction);
                    }
                }
            }
            Direction::Right => {
                for x in (0..self.get_width()).rev() {
                    for y in 0..self.get_length() {
                        self.move_tile(y, x, direction);
                    }
                }
            }
            Direction::Up => {
                for y in 0..self.get_length() {
                    for x in 0..self.get_width() {
                        self.move_tile(y, x, direction);
                    }
                }
            }
            Direction::Down => {
                for y in (0..self.get_length()).rev() {
                    for x in 0..self.get_width() {
                        self.move_tile(y, x, direction);
                    }
                }
            }
        };

        self.clear_connected_tiles();
    }

    fn move_tile(&mut self, y: usize, x: usize, direction: Direction) {
        let (ty, tx) = match direction {
            Direction::Left => (y, x.wrapping_sub(1)),  // (y, x - 1)
            Direction::Right => (y, x.wrapping_add(1)), // (y, x + 1)
            Direction::Up => (y.wrapping_sub(1), x),    // (y - 1, x)
            Direction::Down => (y.wrapping_add(1), x),  // (y + 1, x)
        };

        let from = self.data.get((y, x)).unwrap();
        let Some(to) = self.data.get((ty, tx)) else {
            // Hit the wall
            return;
        };

        // Target is regular or blocker tile: cannot move
        // Origin is empty or blocker tile: cannot move
        if let (Tile::Regular(_), Tile::Empty) = (from, to) {
            self.data.swap((y, x), (ty, tx));
        }
    }

    fn clear_connected_tiles(&mut self) {
        let (length, width) = (self.get_length(), self.get_width());
        let mut uf = QuickUnionUf::<UnionBySize>::new(length * width);
        for ((y, x), tile) in self.data.indexed_iter() {
            let Tile::Regular(color) = tile else { continue };
            let index = y * width + x;

            if x + 1 < width {
                if let Tile::Regular(right_color) = self.data.get((y, x + 1)).unwrap() {
                    if color == right_color {
                        uf.union(index, index + 1);
                    }
                }
            }
            if y + 1 < length {
                if let Tile::Regular(bottom_color) = self.data.get((y + 1, x)).unwrap() {
                    if color == bottom_color {
                        uf.union(index, index + width);
                    }
                }
            }
        }

        for ((y, x), tile) in self.data.indexed_iter_mut() {
            let index = y * width + x;
            let root_index = uf.find(index);
            if uf.get(root_index).size() >= 4 {
                *tile = Tile::Empty;
            }
        }
    }

    pub fn to_ron(&self) -> String {
        let pretty_config = PrettyConfig::new().depth_limit(2);
        ron::ser::to_string_pretty(&self, pretty_config).unwrap()
    }

    pub fn from_ron(ron: &str) -> Result<Self, Box<dyn Error>> {
        Ok(ron::de::from_str(ron)?)
    }
}
