use std::{
    fmt::{Display, Formatter, *},
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy)]
pub struct Timer {
    pub target_ticks: u32,
    pub target_delta: Duration,
    pub last_tick: Instant,
    pub accumulated_delta: Duration,
    pub has_ticked: bool,
}

impl Timer {
    pub(crate) fn new(ticks_per_second: u32) -> Timer {
        let (target_seconds, target_nanos) = match ticks_per_second {
            0 => (std::u64::MAX, 0),
            1 => (1, 0),
            _ => (0, ((1.0 / ticks_per_second as f64) * 1e9) as u32),
        };

        Timer {
            target_ticks: ticks_per_second,
            target_delta: Duration::new(target_seconds, target_nanos),
            last_tick: Instant::now(),
            accumulated_delta: Duration::from_secs(0),
            has_ticked: false,
        }
    }

    pub(crate) fn update(&mut self) {
        let now = Instant::now();
        let diff = now - self.last_tick;

        self.last_tick = now;
        self.accumulated_delta += diff;
        self.has_ticked = false;
    }

    pub(crate) fn tick(&mut self) -> bool {
        if self.accumulated_delta >= self.target_delta {
            self.accumulated_delta -= self.target_delta;
            self.has_ticked = true;

            true
        } else {
            false
        }
    }

    pub(crate) fn set_ticks_per_second(&mut self, ticks_per_second: u32) {
        let (target_seconds, target_nanos) = match ticks_per_second {
            0 => (std::u64::MAX, 0),
            1 => (1, 0),
            _ => (0, ((1.0 / ticks_per_second as f64) * 1e9) as u32),
        };

        self.target_ticks = ticks_per_second;
        self.target_delta = Duration::new(target_seconds, target_nanos);
    }

    pub fn next_tick_proximity(&self) -> f32 {
        let delta = self.accumulated_delta;

        self.target_ticks as f32 * (delta.as_secs() as f32 + (delta.subsec_micros() as f32 / 1_000_000.0))
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new(60)
    }
}

impl Display for Timer {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "[interval]: {}ms, [delta]: {}ms [ticked?]: {}",
            self.target_delta.as_millis(),
            self.accumulated_delta.as_millis(),
            self.has_ticked
        )
    }
}