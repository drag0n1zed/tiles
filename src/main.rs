#[derive(Clone, Copy, Debug)]
enum Tile {
    Empty,
    Blocker,
    Regular(u32), // color
}

struct Grid {
    width: usize,
    length: usize,
    data: Vec<Tile>,
    // data[x * length + width]. Upper left is [0, 0].
}

impl Grid {
    fn new(width: usize, length: usize) -> Self {
        Self {
            width,
            length,
            data: vec![Tile::Empty; width * length],
        }
    }
    fn index(&self, x: usize, y: usize) -> usize {
        x * self.length + y
    }

    fn get(&self, x: usize, y: usize) -> Tile {
        let i = self.index(x, y);
        self.data[i]
    }

    fn set(&mut self, x: usize, y: usize, tile: Tile) {
        let i = self.index(x, y);
        self.data[i] = tile;
    }

    // Iterate order: iterate through every column from top to bottom, from left to right
    // [0, 0] ... [0, l - 1],
    // [1, 0] ... [1, l - 1],
    // [2, 0] ... [2, l - 1],
    // ...
    // [w - 1, 0] ... [w - 1, l - 1],
    fn move_grid_left(&mut self) {
        for x in 0..self.width {
            for y in 0..self.length {
                self.move_tile_left(x, y);
            }
        }
    }

    fn move_tile_left(&mut self, x: usize, y: usize) {
        if x == 0 {
            return;
        }

        let from = self.index(x, y);
        let to = self.index(x - 1, y);
        let mut move_list: Vec<(usize, usize)> = Vec::new();

        // Target is regular or blocker tile or the wall: cannot move
        // Origin is empty or blocker tile: cannot move
        if let (Tile::Empty, Tile::Regular(_)) = (self.data[to], self.data[from]) {
            move_list.push((from, to));
        }

        for (from, to) in move_list {
            self.data.swap(from, to); // (Empty, Regular) -> (Regular, Empty)
        }
    }
}

fn main() {
    let mut grid = Grid::new(5, 5);
    grid.set(2, 0, Tile::Regular(0x11223344));
    grid.set(1, 0, Tile::Blocker);
    grid.move_grid_left();
    println!("{:?}", grid.get(1, 0));
}
