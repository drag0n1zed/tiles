use ratatui::layout::Rect;

pub struct GridLayout {
    rect_lookup: Vec<Rect>,
    grid_width: usize,
}

impl GridLayout {
    pub fn new(rect_lookup: Vec<Rect>, grid_width: usize) -> Self {
        Self {
            rect_lookup,
            grid_width,
        }
    }

    pub fn get_rect_from_coords(&self, (y, x): &(usize, usize)) -> Rect {
        self.rect_lookup[y * self.grid_width + x]
    }
}
