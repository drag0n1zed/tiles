use ndarray::Array2;
use serde::{Deserialize, Serialize};

use super::{Grid, tile::Tile};

#[derive(Serialize, Deserialize)]
pub struct VecGrid {
    steps: usize,
    data: Vec<Vec<Tile>>,
    height: usize,
    width: usize,
}

impl From<Grid> for VecGrid {
    fn from(grid: Grid) -> Self {
        let array = grid.tiles.rows().into_iter().map(|chunk| chunk.to_vec()).collect();

        VecGrid {
            steps: grid.steps,
            data: array,
            width: grid.get_width(),
            height: grid.get_height(),
        }
    }
}

impl From<VecGrid> for Grid {
    fn from(vec_grid: VecGrid) -> Self {
        let vec_flat: Vec<Tile> = vec_grid.data.into_iter().flatten().collect();
        Grid {
            tiles: Array2::from_shape_vec((vec_grid.height, vec_grid.width), vec_flat).unwrap(),
            steps: vec_grid.steps,
            active_animations: Vec::new(),
            pending_pop: false,
        }
    }
}
