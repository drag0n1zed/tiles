use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use crate::grid::{MoveDir, grid_layout::GridLayout, tile::Tile};

pub struct AnimationWidget<'a> {
    pub anim: &'a Animation,
    pub grid_layout: &'a GridLayout,
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
                tile.with_moving_progress(from_rect, to_rect, *direction, t)
                    .render(Default::default(), buf);
            }
            Animation::Clearing { tile, at, .. } => {
                let at_rect = self.grid_layout.get_rect_from_coords(*at);

                let t = 1.0 - self.anim.get_quadratic_out_progress();
                tile.with_clearing_progress(at_rect, t).render(Default::default(), buf);
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum Animation {
    Moving {
        tile: Tile,
        from: (usize, usize),
        direction: MoveDir,
        start_time: Instant,
    },
    Clearing {
        tile: Tile,
        at: (usize, usize),
        start_time: Instant,
    },
}

impl Animation {
    pub fn duration(&self) -> Duration {
        match self {
            Animation::Moving { .. } => Duration::from_millis(300),
            Animation::Clearing { .. } => Duration::from_millis(150),
        }
    }

    // Returns Some containing target tile grid coordinates if self is Animation::Moving, returns None otherwise
    pub fn get_target(&self) -> Option<(usize, usize)> {
        let Animation::Moving { from, direction, .. } = self else {
            return None;
        };
        let (y, x) = *from;
        Some(match direction {
            MoveDir::Up => (y.saturating_sub(1), x),
            MoveDir::Down => (y.saturating_add(1), x),
            MoveDir::Left => (y, x.saturating_sub(1)),
            MoveDir::Right => (y, x.saturating_add(1)),
        })
    }
    pub fn with_layout<'a>(&'a self, grid_layout: &'a GridLayout) -> AnimationWidget<'a> {
        AnimationWidget {
            anim: self,
            grid_layout,
        }
    }

    pub fn is_active(&self) -> bool {
        Instant::now() - self.get_start_time() < self.duration()
    }

    pub fn get_coords(&self) -> HashSet<(usize, usize)> {
        match &self {
            Animation::Moving { from: coord, .. } | Animation::Clearing { at: coord, .. } => {
                [Some(*coord), self.get_target()].into_iter().flatten().collect()
            }
        }
    }

    fn get_start_time(&self) -> Instant {
        match self {
            Animation::Moving { start_time, .. } | Animation::Clearing { start_time, .. } => *start_time,
        }
    }

    fn get_progress(&self) -> f64 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.get_start_time());
        let total_duration = self.duration();
        (elapsed.as_secs_f64() / total_duration.as_secs_f64()).clamp(0.0, 1.0)
    }

    fn get_quartic_out_progress(&self) -> f64 {
        1.0 - (1.0 - self.get_progress()).powi(4)
    }

    fn get_quadratic_out_progress(&self) -> f64 {
        let t = self.get_progress();
        t * (2.0 - t)
    }
}
