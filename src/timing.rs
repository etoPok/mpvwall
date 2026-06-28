use std::time::{Duration, Instant};

pub struct Timing {
    start_time: Instant,
    time_base: f64,
    drop_threshold: Duration,
}

impl Timing {
    pub fn new(time_base: f64) -> Self {
        Self {
            start_time: Instant::now(),
            time_base,
            drop_threshold: Duration::from_millis(50),
        }
    }

    pub fn render_time(&self, pts: i64) -> Instant {
        let secs = pts as f64 * self.time_base;
        self.start_time + Duration::from_secs_f64(secs)
    }

    pub fn should_drop(&self, pts: i64, now: Instant) -> bool {
        let render_time = self.render_time(pts);
        now > render_time + self.drop_threshold
    }

    pub fn sleep_until(render_time: Instant) {
        let now = Instant::now();
        if render_time > now {
            std::thread::sleep(render_time - now);
        }
    }

    pub fn reset(&mut self) {
        self.start_time = Instant::now();
    }

    pub fn set_drop_threshold(&mut self, threshold: Duration) {
        self.drop_threshold = threshold;
    }
}
