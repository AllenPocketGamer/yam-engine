use std::{
    fmt,
    time::{Duration, Instant},
};

use colored::Colorize;

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

/// The untility to get time information.
///
/// You CANNOT constrcutor or modify `Time`, it just only provide the time information about main-loop of engine to you.
///
/// If you want a time utility to diagnostic the time cost of a piece of code, then `ProfileTimer` may meets your needs.
#[derive(Clone, Copy)]
pub struct Time {
    pt: ProfileTimer,

    // Σ[(fps - fps_avg)^2]
    fps_diff_pow: u64,
}

impl Time {
    pub(crate) fn now() -> Self {
        Self {
            pt: Default::default(),

            fps_diff_pow: Default::default(),
        }
    }

    pub(crate) fn begin_record(&mut self) {
        self.pt.begin_record();
    }

    pub(crate) fn finish_record(&mut self) {
        self.pt.finish_record();

        self.fps_diff_pow += f32::powi(self.fps() - self.fps_avg(), 2) as u64;
    }

    pub(crate) fn tick(&mut self) {
        self.finish_record();
        self.begin_record();
    }

    pub fn delta(&self) -> Duration {
        self.pt.delta()
    }

    /// Average of deltas.
    pub fn delta_avg(&self) -> Duration {
        self.pt.delta_avg()
    }

    /// Standard deviation of deltas.
    pub fn delta_sd(&self) -> Duration {
        self.pt.delta_sd()
    }

    pub fn record_count(&self) -> u64 {
        self.pt.record_count()
    }

    pub fn fps(&self) -> f32 {
        let is_zero = self.delta().as_micros() == 0;

        if !is_zero {
            1.0 / self.delta().as_secs_f32()
        } else {
            f32::NAN
        }
    }

    pub fn fps_avg(&self) -> f32 {
        let is_zero = self.delta_avg().as_micros() == 0;

        if !is_zero {
            1.0 / self.delta_avg().as_secs_f32()
        } else {
            f32::NAN
        }
    }

    pub fn fps_sd(&self) -> f32 {
        f32::sqrt(self.fps_variance())
    }

    pub fn time(&self) -> Duration {
        self.delta_avg() * self.record_count() as u32
    }

    fn fps_variance(&self) -> f32 {
        let tick_count = std::cmp::max(1, self.record_count());

        (self.fps_diff_pow as f64 / tick_count as f64) as f32
    }
}

impl fmt::Debug for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use colored::*;

        write!(
            f,
            "{pt_data} | {fps_data} | {debug_data}",
            pt_data = format_args!("{:?}", self.pt),
            fps_data = format_args!(
                "{}, {}, {}, {}",
                format!("fps: {:>4.1}", self.fps()).red(),
                format!("fps(avg): {:>4.1}", self.fps_avg()).yellow(),
                format!("fps(sd): {:>4.1}", self.fps_sd()).blue(),
                format!("fps(var): {:>4.1}", self.fps_variance()).green(),
            ),
            debug_data = format!("fps_diff_pow: {:>8}", self.fps_diff_pow),
        )
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use colored::*;

        write!(
            f,
            "{pt_data} | {fps_data}",
            pt_data = format_args!("{}", self.pt),
            fps_data = format_args!(
                "{}, {}, {}",
                format!("fps: {:>4.1}", self.fps()).red(),
                format!("fps(avg): {:>4.1}", self.fps_avg()).yellow(),
                format!("fps(sd): {:>4.1}", self.fps_sd()).blue(),
            ),
        )
    }
}

#[derive(Clone, Copy)]
pub struct ProfileTimer {
    is_record: bool,
    begin_tick: Instant,
    record_count: u64,

    delta: Duration,
    delta_avg: Duration,

    // Σ[(delta - delta_avg)^2]
    delta_diff_pow_us: u64,
}

impl ProfileTimer {
    pub fn now() -> Self {
        Self {
            begin_tick: Instant::now(),
            delta: Default::default(),
            delta_avg: Default::default(),
            delta_diff_pow_us: Default::default(),
            record_count: Default::default(),

            is_record: false,
        }
    }

    pub fn begin_record(&mut self) {
        if !self.is_record {
            self.begin_tick = Instant::now();
            self.is_record = true;
        }
    }

    pub fn finish_record(&mut self) {
        if self.is_record {
            let now = Instant::now();

            self.delta = now - self.begin_tick;
            self.delta_avg = (self.delta_avg * self.record_count as u32 + self.delta)
                / (self.record_count as u32 + 1);
            self.record_count += 1;

            let delta_us = self.delta.as_micros() as i64;
            let delta_avg_us = self.delta_avg.as_micros() as i64;

            self.delta_diff_pow_us += i64::pow(delta_us - delta_avg_us, 2) as u64;

            self.is_record = false;
        }
    }

    pub fn delta(&self) -> Duration {
        self.delta
    }

    pub fn delta_avg(&self) -> Duration {
        self.delta_avg
    }

    pub fn delta_sd(&self) -> Duration {
        let delta_sd = f64::sqrt(self.delta_variance_micros() as f64);

        Duration::from_micros(delta_sd as u64)
    }

    pub fn record_count(&self) -> u64 {
        self.record_count
    }

    fn delta_variance_micros(&self) -> u64 {
        let record_count = std::cmp::max(self.record_count, 1);

        self.delta_diff_pow_us / record_count
    }
}

impl Default for ProfileTimer {
    fn default() -> Self {
        Self::now()
    }
}

impl fmt::Debug for ProfileTimer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{record_count} | {delta_data} | {debug_data}",
            record_count = format_args!("record_count: {:<8}", self.record_count),
            delta_data = format_args!(
                "{}, {}, {}, {}",
                format!("delta: {:>4.1}ms", self.delta.as_secs_f32() * 1000.0).red(),
                format!(
                    "delta(avg): {:>4.1}ms",
                    self.delta_avg.as_secs_f32() * 1000.0
                )
                .yellow(),
                format!(
                    "delta(sd): {:>4.1}ms",
                    self.delta_sd().as_secs_f32() * 1000.0
                )
                .blue(),
                format!(
                    "delta(var): {:>6.1}ms",
                    self.delta_variance_micros() as f64 / 1000.0
                )
                .green(),
            ),
            debug_data = format!("delta_diff_pow: {:>8.1}", self.delta_diff_pow_us),
        )
    }
}

impl fmt::Display for ProfileTimer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{record_count} | {delta_data}",
            record_count = format_args!("record_count: {:<8}", self.record_count),
            delta_data = format_args!(
                "{}, {}, {}",
                format!("delta: {:>4.1}ms", self.delta.as_secs_f32() * 1000.0).red(),
                format!(
                    "delta(avg): {:>4.1}ms",
                    self.delta_avg.as_secs_f32() * 1000.0
                )
                .yellow(),
                format!(
                    "delta(sd): {:>4.1}ms",
                    self.delta_sd().as_secs_f32() * 1000.0
                )
                .blue(),
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_time_format() {
        let mut time = Time::now();

        println!("Debug Format: ");
        for _ in 0..20 {
            std::thread::spawn(move || {
                println!("{:?}", time);
            });
            std::thread::sleep(Duration::from_millis(10));
            time.tick();
        }

        println!("Display Format: ");
        for _ in 0..20 {
            std::thread::spawn(move || {
                println!("{}", time);
            });
            std::thread::sleep(Duration::from_millis(10));
            time.tick();
        }
    }

    #[test]
    fn test_profile_timer_format() {
        let mut profile_timer = ProfileTimer::now();

        println!("Debug Format: ");
        for _ in 0..20 {
            std::thread::spawn(move || {
                println!("{:?}", profile_timer);
            });

            profile_timer.begin_record();
            std::thread::sleep(Duration::from_millis(10));
            profile_timer.finish_record();
        }

        println!("Display Format: ");
        for _ in 0..20 {
            std::thread::spawn(move || {
                println!("{}", profile_timer);
            });

            profile_timer.begin_record();
            std::thread::sleep(Duration::from_millis(10));
            profile_timer.finish_record();
        }
    }
}
