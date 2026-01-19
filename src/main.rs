use std::{
    error::Error,
    fs::{File, write},
    io::BufReader,
};

use ndarray::prelude::*;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default)]
enum Tile {
    #[default]
    Empty,
    Blocker,
    Regular(u32), // color
}

#[derive(Serialize, Deserialize)]
struct Grid {
    #[serde(skip)]
    data: Array2<Tile>,
}

#[derive(Serialize, Deserialize)]
struct VecGrid {
    dim: [usize; 2],
    pattern: Vec<Vec<Tile>>,
}

impl Grid {
    fn new(width: usize, length: usize) -> Self {
        Self {
            data: Array2::from_elem((width, length), Tile::Empty),
        }
    }

    fn get_width(&self) -> usize {
        self.data.dim().0
    }

    fn get_length(&self) -> usize {
        self.data.dim().1
    }

    fn get(&self, x: usize, y: usize) -> Option<&Tile> {
        self.data.get((x, y))
    }

    fn set(&mut self, x: usize, y: usize, tile: Tile) -> bool {
        match self.data.get_mut((x, y)) {
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
    fn move_grid(&mut self, direction: Direction) {
        match direction {
            Direction::Left => {
                for x in 0..self.get_width() {
                    for y in 0..self.get_length() {
                        self.move_tile(x, y, direction);
                    }
                }
            }
            Direction::Right => {
                for x in (0..self.get_width()).rev() {
                    for y in 0..self.get_length() {
                        self.move_tile(x, y, direction);
                    }
                }
            }
            Direction::Up => {
                for y in 0..self.get_length() {
                    for x in 0..self.get_width() {
                        self.move_tile(x, y, direction);
                    }
                }
            }
            Direction::Down => {
                for y in (0..self.get_length()).rev() {
                    for x in 0..self.get_width() {
                        self.move_tile(x, y, direction);
                    }
                }
            }
        };
    }

    fn move_tile(&mut self, x: usize, y: usize, direction: Direction) {
        let (tx, ty) = match direction {
            Direction::Left => (x.wrapping_sub(1), y),  // (x - 1, y)
            Direction::Right => (x.wrapping_add(1), y), // (x + 1, y)
            Direction::Up => (x, y.wrapping_sub(1)),    // (x, y + 1)
            Direction::Down => (x, y.wrapping_add(1)),  // (x, y - 1)
        };

        let from = self.data.get((x, y)).unwrap();
        let Some(to) = self.data.get((tx, ty)) else {
            // Hit the wall
            return;
        };

        // Target is regular or blocker tile: cannot move
        // Origin is empty or blocker tile: cannot move
        if let (Tile::Regular(_), Tile::Empty) = (from, to) {
            self.data.swap((x, y), (tx, ty));
        }
    }

    fn to_vec(&self) -> VecGrid {
        VecGrid {
            dim: [self.get_width(), self.get_length()],
            pattern: self
                .data
                .columns()
                .into_iter()
                .map(|row| row.iter().copied().collect())
                .collect(),
        }
    }

    fn from_vec(format: VecGrid) -> Result<Self, Box<dyn Error>> {
        let vec: Vec<Tile> = format.pattern.into_iter().flatten().collect();
        let data = Array2::from_shape_vec((format.dim[0], format.dim[1]), vec)?;
        Ok(Self { data })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut grid = Grid::new(3, 3);
    grid.set(2, 0, Tile::Regular(0));
    grid.set(2, 1, Tile::Regular(1));
    // let file = File::open("grid.json")?;
    // let reader = BufReader::new(file);
    // let vec: VecGrid = serde_json::from_reader(reader)?;
    // let mut grid = Grid::from_vec(vec)?;

    grid.move_grid(Direction::Left);

    let pretty_config = PrettyConfig::new().depth_limit(2);
    let ron = ron::ser::to_string_pretty(&grid.to_vec(), pretty_config)?;
    println!("{}", ron);
    write("grid.ron", ron)?;

    Ok(())
}
