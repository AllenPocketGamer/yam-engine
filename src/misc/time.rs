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

        self.target_ticks as f32
            * (delta.as_secs() as f32 + (delta.subsec_micros() as f32 / 1_000_000.0))
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
    // to store interval between tick()
    delta: Duration,

    // Σ(delta_cur - delta_avg_cur)^2
    delta_diff_pow_ms: u64,
    // Σ(fps_cur - fps_avg)^2
    fps_diff_pow: u64,
    tick_count: u64,
}

impl Time {
    pub(crate) fn now() -> Self {
        let now = Instant::now();

        Self {
            start_tick: now,
            last_tick: now,
            delta: Default::default(),

            delta_diff_pow_ms: Default::default(),
            fps_diff_pow: Default::default(),
            tick_count: Default::default(),
        }
    }

    pub(crate) fn tick(&mut self) {
        let now = Instant::now();

        self.delta = now - self.last_tick;
        self.last_tick = now;
        self.tick_count += 1;

        let delta_ms = self.delta.as_millis() as i64;
        let delta_avg_ms = self.delta_avg().as_millis() as i64;

        self.delta_diff_pow_ms += i64::pow(delta_ms - delta_avg_ms, 2) as u64;

        let fps = self.fps() as i64;
        let fps_avg = self.fps_avg() as i64;

        self.fps_diff_pow += i64::pow(fps - fps_avg, 2) as u64;
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::now();
    }

    pub fn time(&self) -> Duration {
        self.last_tick - self.start_tick
    }

    pub fn delta(&self) -> Duration {
        self.delta
    }

    /// Average of deltas.
    pub fn delta_avg(&self) -> Duration {
        let tick_count = std::cmp::max(1, self.tick_count);

        self.time() / (tick_count as u32)
    }

    /// Variance of deltas.
    pub fn delta_variance(&self) -> Duration {
        Duration::from_millis(self.delta_variance_millis())
    }

    /// Standard deviation of deltas.
    pub fn delta_sd(&self) -> Duration {
        // TODO: fix the precious problem(maybe to use integer sqrt root).
        let delta_sd = f64::sqrt(self.delta_variance_millis() as f64);

        Duration::from_millis(delta_sd as u64)
    }

    pub fn fps(&self) -> u64 {
        if self.tick_count > 0 {
            (1.0 / self.delta.as_secs_f32()).floor() as u64
        } else {
            0
        }
    }

    pub fn fps_avg(&self) -> u64 {
        if self.tick_count > 0 {
            (1.0 / self.delta_avg().as_secs_f32()).floor() as u64
        } else {
            0
        }
    }

    // NOTE: estimate approximately now!
    pub fn fps_variance(&self) -> u64 {
        let tick_count = std::cmp::max(1, self.tick_count);

        self.fps_diff_pow / tick_count
    }

    pub fn fps_sd(&self) -> u64 {
        f64::sqrt(self.fps_variance() as f64).floor() as u64
    }

    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }

    pub fn delta_variance_millis(&self) -> u64 {
        let tick_count = std::cmp::max(1, self.tick_count);

        self.delta_diff_pow_ms / tick_count
    }
}

impl fmt::Debug for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use colored::*;

        write!(
            f,
            "{tick_count} | {delta_data} | {fps_data} | {debug_data}",
            tick_count = format_args!("tick_count: {:<8}", self.tick_count),
            delta_data = format_args!(
                "{}, {}, {}, {}",
                format!("delta: {:>4.1}ms", self.delta.as_secs_f32() * 1000.0).red(),
                format!(
                    "delta(avg): {:>4.1}ms",
                    self.delta_avg().as_secs_f32() * 1000.0
                )
                .yellow(),
                format!(
                    "delta(sd): {:>4.1}ms",
                    self.delta_sd().as_secs_f32() * 1000.0
                )
                .blue(),
                format!(
                    "delta(var): {:>4.1}ms",
                    self.delta_variance().as_secs_f32() * 1000.0
                )
                .green(),
            ),
            fps_data = format_args!(
                "{}, {}, {}, {}",
                format!("fps: {:>3}", self.fps()).red(),
                format!("fps(avg): {:>3}", self.fps_avg()).yellow(),
                format!("fps(sd): {:>3}", self.fps_sd()).blue(),
                format!("fps(var): {:>3}", self.fps_variance()).green(),
            ),
            debug_data = format_args!(
                "{}, {}",
                format!("delta_diff_pow: {:>8}ms", self.delta_diff_pow_ms).red(),
                format!("fps_diff_pow: {:>8}", self.fps_diff_pow).yellow(),
            ),
        )
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use colored::*;

        write!(
            f,
            "{tick_count} | {delta_data} | {fps_data}",
            tick_count = format_args!("tick_count: {:<8}", self.tick_count),
            delta_data = format_args!(
                "{}, {}, {}, {}",
                format!("delta: {:>4.1}ms", self.delta.as_secs_f32() * 1000.0).red(),
                format!(
                    "delta(avg): {:>4.1}ms",
                    self.delta_avg().as_secs_f32() * 1000.0
                )
                .yellow(),
                format!(
                    "delta(sd): {:>4.1}ms",
                    self.delta_sd().as_secs_f32() * 1000.0
                )
                .blue(),
                format!(
                    "delta(var): {:>4.1}ms",
                    self.delta_variance().as_secs_f32() * 1000.0
                )
                .green(),
            ),
            fps_data = format_args!(
                "{}, {}, {}, {}",
                format!("fps: {:>3}", self.fps()).red(),
                format!("fps(avg): {:>3}", self.fps_avg()).yellow(),
                format!("fps(sd): {:>3}", self.fps_sd()).blue(),
                format!("fps(var): {:>3}", self.fps_variance()).green(),
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Time;
    use std::time::Duration;

    #[test]
    fn test_time() {
        let mut time = Time::now();

        for _ in 0..100 {
            println!("{:?}", time);
            std::thread::sleep(Duration::from_millis(10));
            time.tick();
        }
    }
}
