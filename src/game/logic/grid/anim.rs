use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use super::{MoveDir, tile::Tile};

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

    pub fn get_progress(&self) -> f64 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.get_start_time());
        let total_duration = self.duration();
        (elapsed.as_secs_f64() / total_duration.as_secs_f64()).clamp(0.0, 1.0)
    }

    pub fn get_quartic_out_progress(&self) -> f64 {
        1.0 - (1.0 - self.get_progress()).powi(4)
    }

    pub fn get_quadratic_out_progress(&self) -> f64 {
        let t = self.get_progress();
        t * (2.0 - t)
    }
}
