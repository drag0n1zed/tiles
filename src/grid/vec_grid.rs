use ndarray::Array2;
use serde::{Deserialize, Serialize};

use crate::grid::{Grid, tile::Tile};

#[derive(Serialize, Deserialize)]
pub struct VecGrid {
    height: usize,
    width: usize,
    data: Vec<Vec<Tile>>,
}

impl From<Grid> for VecGrid {
    fn from(grid: Grid) -> Self {
        let pattern = grid
            .data
            .rows()
            .into_iter()
            .map(|chunk| chunk.to_vec())
            .collect();

        VecGrid {
            width: grid.get_width(),
            height: grid.get_height(),
            data: pattern,
        }
    }
}

impl From<VecGrid> for Grid {
    fn from(vec_grid: VecGrid) -> Self {
        let vec_flat: Vec<Tile> = vec_grid.data.into_iter().flatten().collect();
        Grid {
            data: Array2::from_shape_vec((vec_grid.height, vec_grid.width), vec_flat).unwrap(),
            active_animations: vec![],
            animation_mask: Array2::from_elem((vec_grid.height, vec_grid.width), false),
            pending_clear: false,
        }
    }
}
