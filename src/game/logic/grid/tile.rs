use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use uid::Id;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Tile {
    Empty,
    Blocker,
    Regular {
        #[serde(skip)]
        id: Id<Tile>,
        color: Color,
    },
}
