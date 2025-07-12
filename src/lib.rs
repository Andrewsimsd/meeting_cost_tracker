
//! # Meeting Cost Tracker
//!
//! A library for tracking the cost of meetings in real-time, based on attendee salaries.
//! Designed for integration with TUI frontends using `ratatui`.
//!
//! ## Example
//!
//! ```rust
//! use meeting_cost_tracker::{EmployeeCategory, Meeting};
//!
//! let category = EmployeeCategory::new("Engineer", 120_000.0).unwrap();
//! let mut meeting = Meeting::new();
//! meeting.add_attendee(category.clone(), 3);
//! meeting.start();
//! std::thread::sleep(std::time::Duration::from_millis(500));
//! meeting.stop();
//! println!("Cost: ${:.2}", meeting.total_cost());
//! ```

mod model;
mod meeting;
mod storage;

pub use model::EmployeeCategory;
pub use meeting::Meeting;
pub use storage::{load_categories, save_categories};