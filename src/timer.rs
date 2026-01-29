use std::time::{Duration, Instant};

pub struct Timer {
    last_run: Instant,
    interval: Duration,
}

impl Timer {
    pub fn new(interval: Duration) -> Self {
        Self {
            last_run: Instant::now(),
            interval,
        }
    }
    pub fn ready(&mut self) -> bool {
        if self.last_run.elapsed() >= self.interval {
            self.last_run = Instant::now();
            true
        } else {
            false
        }
    }

    pub fn time_until_ready(&self) -> Duration {
        self.interval.saturating_sub(self.last_run.elapsed())
    }
}
