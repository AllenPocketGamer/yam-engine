use std::{
    fmt,
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
pub struct PulseTimer {
    target_ticks: u32,
    target_delta: Duration,

    last_update: Instant,

    prev_tick: Instant,
    last_tick: Instant,
    total_time: Duration,
    accumulated_time: Duration,
}

impl PulseTimer {
    pub fn new(ticks_per_second: u32) -> PulseTimer {
        let (target_seconds, target_nanos) = match ticks_per_second {
            0 => (std::u64::MAX, 0),
            1 => (1, 0),
            _ => (0, ((1.0 / ticks_per_second as f64) * 1e9) as u32),
        };

        PulseTimer {
            target_ticks: ticks_per_second,
            target_delta: Duration::new(target_seconds, target_nanos),

            last_update: Instant::now(),

            prev_tick: Instant::now(),
            last_tick: Instant::now(),
            total_time: Duration::new(0, 0),
            accumulated_time: Duration::from_secs(0),
        }
    }

    pub fn update(&mut self) -> bool {
        let now = Instant::now();
        let diff = now - self.last_update;

        self.last_update = now;
        self.total_time += diff;
        self.accumulated_time += diff;

        if self.accumulated_time >= self.target_delta {
            self.prev_tick = self.last_tick;
            self.last_tick = self.last_update;

            self.accumulated_time -= self.target_delta;

            true
        } else {
            false
        }
    }

    pub fn delta(&self) -> Duration {
        self.last_tick - self.prev_tick
    }

    pub fn total_time(&self) -> Duration {
        self.total_time
    }
    
    pub fn accumulated_time(&self) -> Duration {
        self.accumulated_time
    }

    pub fn ticks_per_second(&self) -> u32 {
        self.target_ticks
    }

    pub fn next_tick_proximity(&self) -> f32 {
        let delta = self.accumulated_time;

        self.target_ticks as f32 * (delta.as_secs() as f32 + (delta.subsec_micros() as f32 / 1_000_000.0))
    }

    pub fn set_ticks_per_second(&mut self, ticks_per_second: u32) {
        let (target_seconds, target_nanos) = match ticks_per_second {
            0 => (std::u64::MAX, 0),
            1 => (1, 0),
            _ => (0, ((1.0 / ticks_per_second as f64) * 1e9) as u32),
        };

        self.target_ticks = ticks_per_second;
        self.target_delta = Duration::new(target_seconds, target_nanos);
    }
}

impl Default for PulseTimer {
    fn default() -> Self {
        Self::new(60)
    }
}

impl fmt::Debug for PulseTimer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PulseTimer")
        .field("frequency", &self.target_ticks)
        .field("delta", &self.delta().as_secs_f32())
        .field("total_time", &self.total_time.as_secs_f32())
        .field("accumulated_time", &self.accumulated_time.as_secs_f32())
        .field("next_tick_proximity", &self.next_tick_proximity())
        .finish()
    }
}

// FIXME: maybe it will force threads running from parallel to serial when accessing it by mutability
// try to fix it, prevent legion to reschedule parallel systems
#[derive(Clone, Copy)]
pub struct TickTimer {
    last_tick: Instant,
    delta: Duration,
}

impl TickTimer {
    pub fn new() -> Self {
        Self {
            last_tick: Instant::now(),
            delta: Default::default(),
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();

        self.delta = now - self.last_tick;
        self.last_tick = now;
    }

    pub fn delta(&self) -> Duration {
        self.delta
    }
}

impl Default for TickTimer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
pub struct Time {
    start_tick: Instant,
    last_tick: Instant,
    delta: Duration,
    tick_count: u32,
}

impl Time {
    pub(crate) fn now() -> Self {
        Self {
            last_tick: Instant::now(),
            start_tick: Instant::now(),
            delta: Default::default(),
            tick_count: 0,
        }
    }

    pub(crate) fn tick(&mut self) {
        let now = Instant::now();

        self.delta = now - self.last_tick;
        self.last_tick = now;
        self.tick_count += 1;
    }

    pub(crate) fn reset(&mut self) {
        self.start_tick = Instant::now();
        self.last_tick = Instant::now();
        self.delta = Default::default();
        self.tick_count = 0;
    }

    pub fn time(&self) -> Duration {
        self.last_tick - self.start_tick
    }

    pub fn delta(&self) -> Duration {
        self.delta
    }

    pub fn tick_count(&self) -> u32 {
        self.tick_count
    }
}

impl fmt::Debug for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FrameTimer")
        .field("time", &self.time().as_secs_f32())
        .field("delta", &self.delta.as_secs_f32())
        .field("tick count", &self.tick_count)
        .finish()
    }
}