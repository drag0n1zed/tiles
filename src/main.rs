use ndarray::prelude::*;

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug)]
enum Tile {
    Empty,
    Blocker,
    Regular(u32), // color
}

struct Grid {
    width: usize,
    length: usize,
    data: Array2<Tile>,
    // data[x * length + width]. Upper left is [0, 0].
}

impl Grid {
    fn new(width: usize, length: usize) -> Self {
        Self {
            width,
            length,
            data: Array2::from_elem((width, length), Tile::Empty),
        }
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
                for x in 0..self.width {
                    for y in 0..self.length {
                        self.move_tile(x, y, direction);
                    }
                }
            }
            Direction::Right => {
                for x in (0..self.width).rev() {
                    for y in 0..self.length {
                        self.move_tile(x, y, direction);
                    }
                }
            }
            Direction::Up => {
                for y in 0..self.length {
                    for x in 0..self.width {
                        self.move_tile(x, y, direction);
                    }
                }
            }
            Direction::Down => {
                for y in (0..self.length).rev() {
                    for x in 0..self.width {
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
}

fn main() {
    let mut grid = Grid::new(3, 3);
    grid.set(2, 0, Tile::Regular(0));
    grid.set(2, 1, Tile::Regular(1));

    grid.move_grid(Direction::Down);
    println!("{:?}", grid.get(2, 0).unwrap());
    println!("{:?}", grid.get(2, 1).unwrap());
    println!("{:?}", grid.get(2, 2).unwrap());
}
