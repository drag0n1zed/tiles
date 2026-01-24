use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use std::time::{Duration, Instant};

use crate::grid::tile::Tile;

#[derive(Clone, Copy)]
pub enum Animation {
    Moving {
        tile: Tile,
        from: (usize, usize),
        to: (usize, usize),
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
            Animation::Moving { .. } => Duration::from_millis(200),
            Animation::Clearing { .. } => Duration::from_millis(100),
        }
    }

    pub fn render_moving(&self, from_rect: Rect, to_rect: Rect, buf: &mut Buffer) {
        let Animation::Moving { tile, .. } = self else {
            return;
        };

        let t = self.get_quartic_out_progress();
        let start_x = from_rect.x as f64;
        let start_y = from_rect.y as f64;
        let end_x = to_rect.x as f64;
        let end_y = to_rect.y as f64;

        let current_x = start_x + (end_x - start_x) * t;
        let current_y = start_y + (end_y - start_y) * t;

        let current_rect = Rect::new(
            current_x.round() as u16,
            current_y.round() as u16,
            from_rect.width,
            from_rect.height,
        );

        tile.render(current_rect, buf);
    }

    pub fn render_clearing(&self, at_rect: Rect, buf: &mut Buffer) {
        let Animation::Clearing { tile, .. } = self else {
            return;
        };

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
