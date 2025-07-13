use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::model::EmployeeCategory;

/// Tracks a running meeting's configuration and state.
#[derive(Debug)]
pub struct Meeting {
    attendees: HashMap<EmployeeCategory, u32>,
    start_time: Option<Instant>,
    elapsed: Duration,
    running: bool,
}

impl Meeting {
    /// Creates a new, empty [`Meeting`].
    ///
    /// # Arguments
    ///
    /// * None
    ///
    /// # Returns
    ///
    /// A new empty [`Meeting`].
    ///
    /// ## Example
    /// ```
    /// use meeting_cost_tracker::Meeting;
    /// let meeting = Meeting::new();
    /// assert!(!meeting.is_running());
    /// ```
    ///
    /// # See Also
    /// * [`Meeting::start`]
    /// * [`Meeting::add_attendee`]
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
    ///
    /// ## Example
    /// ```
    /// use meeting_cost_tracker::{EmployeeCategory, Meeting};
    /// let mut meeting = Meeting::new();
    /// let cat = EmployeeCategory::new("Engineer", 100_000).unwrap();
    /// meeting.add_attendee(&cat, 3);
    /// ```
    ///
    /// # Arguments
    ///
    /// * `category` - The [`EmployeeCategory`] to add.
    /// * `count` - Number of attendees to add.
    ///
    /// # Returns
    ///
    /// Nothing.
    ///
    /// # See Also
    /// * [`Meeting::remove_attendee`]
    pub fn add_attendee(&mut self, category: &EmployeeCategory, count: u32) {
        let entry = self.attendees.entry(category.clone()).or_insert(0);
        *entry += count;
    }

    /// Removes up to `count` attendees of the given title from the meeting.
    /// If the resulting count is zero, the attendee entry is removed entirely.
    ///
    /// ## Example
    /// ```
    /// use meeting_cost_tracker::{EmployeeCategory, Meeting};
    /// let cat = EmployeeCategory::new("Dev", 90_000).unwrap();
    /// let mut meeting = Meeting::new();
    /// meeting.add_attendee(&cat, 2);
    /// meeting.remove_attendee("Dev", 1);
    /// ```
    ///
    /// # Arguments
    ///
    /// * `title` - Title of the attendees to remove.
    /// * `count` - Number of attendees to remove.
    ///
    /// # Returns
    ///
    /// Nothing.
    ///
    /// # See Also
    /// * [`Meeting::add_attendee`]
    pub fn remove_attendee(&mut self, title: &str, count: u32) {
        if let Some(key) = self.attendees.keys().find(|c| c.title() == title).cloned() {
            if let Some(entry) = self.attendees.get_mut(&key) {
                if *entry <= count {
                    self.attendees.remove(&key);
                } else {
                    *entry -= count;
                }
            }
        }
    }

    /// Returns an iterator over the attendee list.
    ///
    /// The iterator yields the employee title along with a tuple `(salary, count)`.
    ///
    /// ## Example
    /// ```
    /// use meeting_cost_tracker::{EmployeeCategory, Meeting};
    /// let cat = EmployeeCategory::new("QA", 80_000).unwrap();
    /// let mut meeting = Meeting::new();
    /// meeting.add_attendee(&cat, 1);
    /// for (cat, count) in meeting.attendees() {
    ///     println!("{} - {} x ${}", cat.title(), count, cat.salary());
    /// }
    /// ```
    ///
    /// # Arguments
    ///
    /// * None
    ///
    /// # Returns
    ///
    /// An iterator over `(EmployeeCategory, count)` entries.
    ///
    /// # See Also
    /// * [`Meeting::add_attendee`]
    /// * [`Meeting::remove_attendee`]
    pub fn attendees(&self) -> impl Iterator<Item = (&EmployeeCategory, &u32)> {
        self.attendees.iter()
    }

    /// Starts the meeting timer.
    ///
    /// Calling this method while the meeting is already running has no effect.
    ///
    /// ## Example
    /// ```
    /// use meeting_cost_tracker::Meeting;
    /// let mut meeting = Meeting::new();
    /// meeting.start();
    /// ```
    ///
    /// # Arguments
    ///
    /// * None
    ///
    /// # Returns
    ///
    /// Nothing.
    ///
    /// # See Also
    /// * [`Meeting::stop`]
    pub fn start(&mut self) {
        if !self.running {
            self.start_time = Some(Instant::now());
            self.running = true;
        }
    }

    /// Stops the meeting and accumulates elapsed time.
    ///
    /// This method is safe to call multiple times.
    ///
    /// ## Example
    /// ```
    /// use meeting_cost_tracker::Meeting;
    /// let mut meeting = Meeting::new();
    /// meeting.start();
    /// meeting.stop();
    /// ```
    ///
    /// # Arguments
    ///
    /// * None
    ///
    /// # Returns
    ///
    /// Nothing.
    ///
    /// # See Also
    /// * [`Meeting::start`]
    pub fn stop(&mut self) {
        if self.running {
            if let Some(start_time) = self.start_time.take() {
                self.elapsed += start_time.elapsed();
            }
            self.running = false;
        }
    }

    /// Resets the meeting to its initial state.
    ///
    /// This clears all attendees and elapsed time.
    ///
    /// ## Example
    /// ```
    /// use meeting_cost_tracker::{EmployeeCategory, Meeting};
    /// let mut meeting = Meeting::new();
    /// let cat = EmployeeCategory::new("Engineer", 120_000).unwrap();
    /// meeting.add_attendee(&cat, 2);
    /// meeting.reset();
    /// assert_eq!(meeting.total_cost(), 0.0);
    /// ```
    ///
    /// # Arguments
    ///
    /// * None
    ///
    /// # Returns
    ///
    /// Nothing.
    ///
    /// # See Also
    /// * [`Meeting::start`]
    /// * [`Meeting::stop`]
    pub fn reset(&mut self) {
        self.attendees.clear();
        self.start_time = None;
        self.elapsed = Duration::ZERO;
        self.running = false;
    }

    /// Returns the total duration the meeting has been active.
    ///
    /// ## Example
    /// ```
    /// use meeting_cost_tracker::Meeting;
    /// let mut meeting = Meeting::new();
    /// meeting.start();
    /// std::thread::sleep(std::time::Duration::from_millis(10));
    /// meeting.stop();
    /// let d = meeting.duration();
    /// assert!(d.as_millis() > 0);
    /// ```
    ///
    /// # Arguments
    ///
    /// * None
    ///
    /// # Returns
    ///
    /// The total duration of the meeting.
    ///
    /// # See Also
    /// * [`Meeting::start`]
    /// * [`Meeting::stop`]
    #[must_use]
    pub fn duration(&self) -> Duration {
        self.elapsed + self.current_duration()
    }

    /// Returns the cost in dollars based on elapsed time and attendee salaries.
    ///
    /// ## Example
    /// ```
    /// use meeting_cost_tracker::{EmployeeCategory, Meeting};
    /// let mut meeting = Meeting::new();
    /// let cat = EmployeeCategory::new("Engineer", 100_000).unwrap();
    /// meeting.add_attendee(&cat, 1);
    /// meeting.start();
    /// std::thread::sleep(std::time::Duration::from_millis(10));
    /// meeting.stop();
    /// let cost = meeting.total_cost();
    /// assert!(cost > 0.0);
    /// ```
    ///
    /// # Arguments
    ///
    /// * None
    ///
    /// # Returns
    ///
    /// The total cost in dollars.
    ///
    /// # See Also
    /// * [`Meeting::duration`]
    /// * [`EmployeeCategory::cost_per_millisecond`]
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn total_cost(&self) -> f64 {
        let millis = self.duration().as_millis() as f64;
        self.attendees
            .iter()
            .map(|(cat, count)| cat.cost_per_millisecond() * f64::from(*count) * millis)
            .sum()
    }

    /// Checks whether the meeting is currently running.
    ///
    /// ## Example
    /// ```
    /// use meeting_cost_tracker::Meeting;
    /// let mut meeting = Meeting::new();
    /// meeting.start();
    /// assert!(meeting.is_running());
    /// meeting.stop();
    /// assert!(!meeting.is_running());
    /// ```
    ///
    /// # Arguments
    ///
    /// * None
    ///
    /// # Returns
    ///
    /// `true` if the meeting is running.
    ///
    /// # See Also
    /// * [`Meeting::start`]
    /// * [`Meeting::stop`]
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Computes the duration since the meeting was started if it is running.
    ///
    /// # Arguments
    ///
    /// * None
    ///
    /// # Returns
    ///
    /// The current running duration.
    ///
    /// # See Also
    /// * [`Meeting::duration`]
    fn current_duration(&self) -> Duration {
        if self.running {
            self.start_time.map_or(Duration::ZERO, |t| t.elapsed())
        } else {
            Duration::ZERO
        }
    }
}

impl Default for Meeting {
    fn default() -> Self {
        Self::new()
    }
}
