use std::{error::Error, fmt};

use ndarray::prelude::*;
use ratatui::{
    style::{Color, Style},
    text::Span,
};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub enum Tile {
    #[default]
    Empty,
    Blocker,
    Regular(u8), // color
}

impl From<&Tile> for Span<'static> {
    fn from(tile: &Tile) -> Self {
        match tile {
            Tile::Empty => Span::styled("[ ]", Style::default().fg(Color::White)),
            Tile::Blocker => Span::styled("[#]", Style::default().fg(Color::Black).bold()),
            Tile::Regular(color) => {
                Span::styled("[x]", Style::default().fg(Color::Indexed(color.clone())))
            }
        }
    }
}

pub struct Grid {
    data: Array2<Tile>,
}

#[derive(Serialize, Deserialize)]
pub struct VecGrid {
    length: usize,
    width: usize,
    pub pattern: Vec<Vec<Tile>>,
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

    // Iterate order: iterate through every column from top to bottom, from left to right
    // [0, 0] ... [0, l - 1],
    // [1, 0] ... [1, l - 1],
    // [2, 0] ... [2, l - 1],
    // ...
    // [w - 1, 0] ... [w - 1, l - 1],
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

    pub fn to_vec(&self) -> VecGrid {
        VecGrid {
            length: self.get_length(),
            width: self.get_width(),
            pattern: self
                .data
                .rows()
                .into_iter()
                .map(|row| row.iter().copied().collect())
                .collect(),
        }
    }

    pub fn to_ron(&self) -> String {
        let pretty_config = PrettyConfig::new().depth_limit(2);
        ron::ser::to_string_pretty(&self.to_vec(), pretty_config).unwrap()
    }

    pub fn from_vec(vec_grid: VecGrid) -> Result<Self, Box<dyn Error>> {
        let vec_flat: Vec<Tile> = vec_grid.pattern.into_iter().flatten().collect();
        let data = Array2::from_shape_vec((vec_grid.length, vec_grid.width), vec_flat)?;
        Ok(Self { data })
    }

    pub fn from_ron(ron: &str) -> Result<Self, Box<dyn Error>> {
        let vec: VecGrid = ron::de::from_str(ron)?;
        Grid::from_vec(vec)
    }
}
