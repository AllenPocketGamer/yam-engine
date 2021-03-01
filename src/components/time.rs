#![allow(dead_code)]

use colored::Colorize;

use std::{
    fmt,
    time::{Duration, Instant},
};

/// The untility to get time information.
///
/// You CANNOT constrcutor or modify `Time`, it just only provide the time information about main-loop of engine to you.
///
/// If you want a time utility to diagnose the time cost of a piece of code, then `ProfileTimer` may meets your needs.
#[derive(Clone, Copy)]
pub struct Time {
    pt: DiagnosticTimer,

    // Σ[(fps - fps_avg)^2]
    fps_diff_pow: u64,
}

impl Time {
    /// Create instance of `Time`.
    pub(crate) fn now() -> Self {
        Self {
            pt: DiagnosticTimer::now(),

            fps_diff_pow: Default::default(),
        }
    }

    /// Begin a time record, call `Self::finish_record()` when you want to finish record.
    pub(crate) fn start_record(&mut self) {
        self.pt.start_record();
    }

    /// Finish a time record.
    pub(crate) fn stop_record(&mut self) {
        self.pt.stop_record();

        self.fps_diff_pow += f32::powi(self.fps() - self.fps_avg(), 2) as u64;
    }

    /// Whether `Self::begin_record()` has been called.
    pub(crate) fn is_recording(&self) -> bool {
        self.pt.is_recording()
    }

    /// Finish a time record and begin a new time record.
    ///
    /// Look like press the stopwatch.
    pub(crate) fn tick(&mut self) {
        self.stop_record();
        self.start_record();
    }

    /// The interval between `Self::begin_record()` and `Self::finish_record()`.
    pub fn delta(&self) -> Duration {
        self.pt.delta()
    }

    /// The average of deltas recorded.
    pub fn delta_avg(&self) -> Duration {
        self.pt.delta_avg()
    }

    /// The standard deviation of deltas recorded.
    pub fn delta_sd(&self) -> Duration {
        self.pt.delta_sd()
    }

    /// The count to call pair of `Self::begin_record()` and `Self::finish_record()`
    pub fn record_count(&self) -> u64 {
        self.pt.record_count()
    }

    /// The calculated value of framerate: 1.0 / `Self::delta()`.
    pub fn fps(&self) -> f32 {
        let is_zero = self.delta().as_micros() == 0;

        if !is_zero {
            1.0 / self.delta().as_secs_f32()
        } else {
            f32::NAN
        }
    }

    /// The calculated value of average of framerates: 1.0 / `Self::delta_avg()`.
    pub fn fps_avg(&self) -> f32 {
        let is_zero = self.delta_avg().as_micros() == 0;

        if !is_zero {
            1.0 / self.delta_avg().as_secs_f32()
        } else {
            f32::NAN
        }
    }

    /// The standard deviation of framerates recorded.
    pub fn fps_sd(&self) -> f32 {
        f32::sqrt(self.fps_variance())
    }

    /// The duration from construction to last record.
    pub fn time(&self) -> Duration {
        self.delta_avg() * self.record_count() as u32
    }

    /// The variance of framerates recorded.
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
                format!("fps: {:>6.1}", self.fps()).red(),
                format!("fps(avg): {:>6.1}", self.fps_avg()).yellow(),
                format!("fps(sd): {:>6.1}", self.fps_sd()).blue(),
                format!("fps(var): {:>6.1}", self.fps_variance()).green(),
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
                format!("fps: {:>6.1}", self.fps()).red(),
                format!("fps(avg): {:>6.1}", self.fps_avg()).yellow(),
                format!("fps(sd): {:>6.1}", self.fps_sd()).blue(),
            ),
        )
    }
}

/// A timer used to diagnose the performance of a piece of code.
#[derive(Clone, Copy)]
pub struct DiagnosticTimer {
    is_recording: bool,
    begin_tick: Instant,
    record_count: u64,

    delta: Duration,
    delta_avg: Duration,

    // Σ[(delta - delta_avg)^2]
    delta_diff_pow_us: u64,
}

impl DiagnosticTimer {
    /// Craete instance of `ProfileTimer` and call `Self::begin_record()` automatically.
    pub fn now() -> Self {
        Self {
            is_recording: true,
            begin_tick: Instant::now(),
            record_count: Default::default(),

            delta: Default::default(),
            delta_avg: Default::default(),

            delta_diff_pow_us: Default::default(),
        }
    }

    /// Create instance of `ProfileTimer`.
    pub fn new() -> Self {
        Self {
            is_recording: false,
            begin_tick: Instant::now(),
            record_count: Default::default(),

            delta: Default::default(),
            delta_avg: Default::default(),

            delta_diff_pow_us: Default::default(),
        }
    }

    /// Begin a time record, call `Self::finish_record()` when you want to finish record.
    pub fn start_record(&mut self) {
        if !self.is_recording {
            self.begin_tick = Instant::now();
            self.is_recording = true;
        }
    }

    /// Finish a time record.
    pub fn stop_record(&mut self) {
        if self.is_recording {
            let now = Instant::now();

            self.delta = now - self.begin_tick;
            self.delta_avg = (self.delta_avg * self.record_count as u32 + self.delta)
                / (self.record_count as u32 + 1);
            self.record_count += 1;

            let delta_us = self.delta.as_micros() as i64;
            let delta_avg_us = self.delta_avg.as_micros() as i64;

            self.delta_diff_pow_us += i64::pow(delta_us - delta_avg_us, 2) as u64;

            self.is_recording = false;
        }
    }

    /// Whether `Self::begin_record()` has been called.
    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    /// The interval between `Self::begin_record()` and `Self::finish_record()`.
    pub fn delta(&self) -> Duration {
        self.delta
    }

    /// The average of deltas recorded.
    pub fn delta_avg(&self) -> Duration {
        self.delta_avg
    }

    /// The standard deviation of deltas recorded.
    pub fn delta_sd(&self) -> Duration {
        let delta_sd = f64::sqrt(self.delta_variance_micros() as f64);

        Duration::from_micros(delta_sd as u64)
    }

    /// The count to call pair of `Self::begin_record()` and `Self::finish_record()`
    pub fn record_count(&self) -> u64 {
        self.record_count
    }

    /// The variance of deltas recorded(unit is us^2).
    fn delta_variance_micros(&self) -> u64 {
        let record_count = std::cmp::max(self.record_count, 1);

        self.delta_diff_pow_us / record_count
    }
}

impl Default for DiagnosticTimer {
    /// Craete instance of `ProfileTimer` and call `Self::begin_record()` automatically.
    fn default() -> Self {
        Self::now()
    }
}

impl fmt::Debug for DiagnosticTimer {
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

impl fmt::Display for DiagnosticTimer {
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
    fn test_diagnostic_timer_format() {
        let mut profile_timer = DiagnosticTimer::now();

        println!("Debug Format: ");
        for _ in 0..20 {
            std::thread::spawn(move || {
                println!("{:?}", profile_timer);
            });

            profile_timer.start_record();
            std::thread::sleep(Duration::from_millis(10));
            profile_timer.stop_record();
        }

        println!("Display Format: ");
        for _ in 0..20 {
            std::thread::spawn(move || {
                println!("{}", profile_timer);
            });

            profile_timer.start_record();
            std::thread::sleep(Duration::from_millis(10));
            profile_timer.stop_record();
        }
    }
}
