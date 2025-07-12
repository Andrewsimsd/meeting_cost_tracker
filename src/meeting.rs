use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::model::EmployeeCategory;

/// Tracks a running meeting's configuration and state.
#[derive(Debug)]
pub struct Meeting {
    attendees: HashMap<String, (f64, u32)>, // title -> (salary, count)
    start_time: Option<Instant>,
    elapsed: Duration,
    running: bool,
}

impl Meeting {
    /// Creates a new, empty [`Meeting`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            attendees: HashMap::new(),
            start_time: None,
            elapsed: Duration::ZERO,
            running: false,
        }
    }

    /// Adds `count` attendees of a given [`EmployeeCategory`] to the meeting.
    pub fn add_attendee(&mut self, category: EmployeeCategory, count: u32) {
    let entry = self.attendees.entry(category.title().to_string()).or_insert((category.salary(), 0));
    entry.1 += count;
    }

    /// Starts the meeting. Does nothing if already running.
    pub fn start(&mut self) {
        if !self.running {
            self.start_time = Some(Instant::now());
            self.running = true;
        }
    }

    /// Stops the meeting and accumulates elapsed time. Safe to call multiple times.
    pub fn stop(&mut self) {
        if self.running {
            if let Some(start_time) = self.start_time.take() {
                self.elapsed += start_time.elapsed();
            }
            self.running = false;
        }
    }

    /// Resets the meeting.
    pub fn reset(&mut self) {
        self.attendees.clear();
        self.start_time = None;
        self.elapsed = Duration::ZERO;
        self.running = false;
    }

    /// Returns the total duration the meeting has been active.
    #[must_use]
    pub fn duration(&self) -> Duration {
        self.elapsed + self.current_duration()
    }

    /// Returns the cost in dollars based on elapsed time and attendee salaries.
    #[must_use]
    pub fn total_cost(&self) -> f64 {
        let millis = self.duration().as_millis() as f64;
        self.attendees
            .iter()
            .map(|(_, (salary, count))| {
                let cost_per_ms = salary / (365.25 * 24.0 * 60.0 * 60.0 * 1000.0);
                cost_per_ms * f64::from(*count) * millis
            })
            .sum()
    }

    /// Checks whether the meeting is currently running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.running
    }

    fn current_duration(&self) -> Duration {
        if self.running {
            self.start_time.map_or(Duration::ZERO, |t| t.elapsed())
        } else {
            Duration::ZERO
        }
    }
}
