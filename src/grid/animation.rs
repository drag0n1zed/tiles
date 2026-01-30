use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use std::time::{Duration, Instant};

use crate::grid::{MoveDirection, grid_layout::GridLayout, tile::Tile};

pub struct AnimationWidget<'a> {
    pub anim: &'a Animation,
    pub grid_layout: &'a GridLayout,
}

impl<'a> Widget for AnimationWidget<'a> {
    fn render(self, _rect: Rect, buf: &mut Buffer) {
        match self.anim {
            Animation::Moving {
                tile,
                from,
                direction,
                ..
            } => {
                let to = self.anim.get_target().unwrap();
                let (from_rect, to_rect) = (
                    self.grid_layout.get_rect_from_coords(*from),
                    self.grid_layout.get_rect_from_coords(to),
                );
                self.anim
                    .render_moving(tile, from_rect, to_rect, *direction, buf);
            }
            Animation::Clearing { tile, at, .. } => {
                let at_rect = self.grid_layout.get_rect_from_coords(*at);
                self.anim.render_clearing(tile, at_rect, buf);
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum Animation {
    Moving {
        tile: Tile,
        from: (usize, usize),
        direction: MoveDirection,
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
            Animation::Clearing { .. } => Duration::from_millis(200),
        }
    }

    // Returns Some containing target tile grid coordinates if self is Animation::Moving, returns None otherwise
    pub fn get_target(&self) -> Option<(usize, usize)> {
        let Animation::Moving {
            from, direction, ..
        } = self
        else {
            return None;
        };
        let (y, x) = *from;
        Some(match direction {
            MoveDirection::Up => (y.saturating_sub(1), x),
            MoveDirection::Down => (y.saturating_add(1), x),
            MoveDirection::Left => (y, x.saturating_sub(1)),
            MoveDirection::Right => (y, x.saturating_add(1)),
        })
    }
    pub fn with_layout<'a>(&'a self, grid_layout: &'a GridLayout) -> AnimationWidget<'a> {
        AnimationWidget {
            anim: self,
            grid_layout,
        }
    }

    fn render_moving(
        &self,
        tile: &Tile,
        from_rect: Rect,
        to_rect: Rect,
        direction: MoveDirection,
        buf: &mut Buffer,
    ) {
        let t = self.get_quartic_out_progress();

        let start_x = from_rect.x as f64;
        let start_y = from_rect.y as f64;
        let end_x = to_rect.x as f64;
        let end_y = to_rect.y as f64;

        let current_x = start_x + (end_x - start_x) * t;
        let current_y = start_y + (end_y - start_y) * t;

        let x_int = current_x.floor() as u16;
        let y_int = current_y.floor() as u16;

        let offset_x = current_x.fract();
        let offset_y = current_y.fract();

        let (offset, rect) = match direction {
            MoveDirection::Up | MoveDirection::Down => (
                offset_y,
                Rect::new(from_rect.x, y_int, from_rect.width, from_rect.height),
            ),
            MoveDirection::Left | MoveDirection::Right => (
                offset_x,
                Rect::new(x_int, from_rect.y, from_rect.width, from_rect.height),
            ),
        };

        tile.with_offset(direction, offset).render(rect, buf);
    }

    fn render_clearing(&self, tile: &Tile, at_rect: Rect, buf: &mut Buffer) {
        let t = 1.0 - self.get_quadratic_out_progress();
        let target_width = (at_rect.width as f64 * t).round() as u16;
        let target_height = (at_rect.height as f64 * t).round() as u16;

        if target_width == 0 || target_height == 0 {
            return;
        }

        let offset_x = (at_rect.width.saturating_sub(target_width)) / 2;
        let offset_y = (at_rect.height.saturating_sub(target_height)) / 2;

        let pop_rect = Rect::new(
            at_rect.x + offset_x,
            at_rect.y + offset_y,
            target_width,
            target_height,
        );

        tile.render(pop_rect, buf);
    }

    pub fn is_active(&self) -> bool {
        Instant::now() - self.get_start_time() < self.duration()
    }

    fn get_start_time(&self) -> Instant {
        match self {
            Animation::Moving { start_time, .. } | Animation::Clearing { start_time, .. } => {
                *start_time
            }
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
