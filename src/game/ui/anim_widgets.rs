mod clearing;
mod moving;

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::game::{
    logic::grid::anim::Animation,
    ui::{
        anim_widgets::{clearing::ClearingTile, moving::MovingTile},
        grid_layout::GridLayout,
    },
};

pub struct AnimationWidget<'a> {
    anim: &'a Animation,
    grid_layout: &'a GridLayout,
}

impl<'a> AnimationWidget<'a> {
    pub fn new(anim: &'a Animation, grid_layout: &'a GridLayout) -> Self {
        Self { anim, grid_layout }
    }
}

impl<'a> Widget for AnimationWidget<'a> {
    fn render(self, _rect: Rect, buf: &mut Buffer) {
        match self.anim {
            Animation::Moving {
                tile, from, direction, ..
            } => {
                let to = self.anim.get_target().unwrap();
                let (from_rect, to_rect) = (
                    self.grid_layout.get_rect_from_coords(*from),
                    self.grid_layout.get_rect_from_coords(to),
                );

                let t = self.anim.get_quartic_out_progress();
                MovingTile::new(tile, from_rect, to_rect, *direction, t).render(Default::default(), buf);
            }
            Animation::Clearing { tile, at, .. } => {
                let at_rect = self.grid_layout.get_rect_from_coords(*at);

                let t = 1.0 - self.anim.get_quadratic_out_progress();
                ClearingTile::new(tile, at_rect, t).render(Default::default(), buf);
            }
        }
    }
}
