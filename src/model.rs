use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Number of milliseconds in a typical work year (2,000 hours).
pub const MILLIS_PER_WORK_YEAR: f64 = 2000.0 * 60.0 * 60.0 * 1000.0;

/// Errors related to employee category validation.
#[derive(Debug, Error)]
pub enum EmployeeCategoryError {
    /// The salary must be greater than zero.
    #[error("Salary must be greater than zero")]
    InvalidSalary,

    /// Title must not be empty.
    #[error("Title must not be empty")]
    EmptyTitle,
}

/// Represents an employee category (e.g., Engineer, Manager) with a yearly salary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EmployeeCategory {
    title: String,
    salary: u64,
}

impl EmployeeCategory {
    /// Creates a new [`EmployeeCategory`] after validating its fields.
    ///
    /// # Errors
    ///
    /// Returns an [`EmployeeCategoryError`] if `title` is empty or `salary` is not
    /// greater than zero.
    ///
    /// ## Example
    ///
    /// ```
    /// use meeting_cost_tracker::EmployeeCategory;
    /// let category = EmployeeCategory::new("Engineer", 120_000).unwrap();
    /// assert_eq!(category.title(), "Engineer");
    /// ```
    ///
    /// # See Also
    /// * [`EmployeeCategory::title`]
    /// * [`EmployeeCategory::salary`]
    ///
    /// # Arguments
    ///
    /// * `title` - Category title.
    /// * `salary` - Annual salary in dollars.
    ///
    /// # Returns
    ///
    /// A new [`EmployeeCategory`] on success.
    pub fn new<T: Into<String>>(title: T, salary: u64) -> Result<Self, EmployeeCategoryError> {
        let title = title.into();
        if title.trim().is_empty() {
            return Err(EmployeeCategoryError::EmptyTitle);
        }
        if salary == 0 {
            return Err(EmployeeCategoryError::InvalidSalary);
        }
        Ok(Self { title, salary })
    }

    /// Returns the title of the employee category.
    ///
    /// ## Example
    /// ```
    /// use meeting_cost_tracker::EmployeeCategory;
    /// let cat = EmployeeCategory::new("Designer", 80_000).unwrap();
    /// assert_eq!(cat.title(), "Designer");
    /// ```
    ///
    /// # Arguments
    ///
    /// * None
    ///
    /// # Returns
    ///
    /// The title string.
    ///
    /// # See Also
    /// * [`EmployeeCategory::salary`]
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the annual salary for the category.
    ///
    /// ## Example
    /// ```
    /// use meeting_cost_tracker::EmployeeCategory;
    /// let cat = EmployeeCategory::new("Engineer", 100_000).unwrap();
    /// assert_eq!(cat.salary(), 100_000);
    /// ```
    ///
    /// # Arguments
    ///
    /// * None
    ///
    /// # Returns
    ///
    /// Annual salary in dollars.
    ///
    /// # See Also
    /// * [`EmployeeCategory::title`]
    /// * [`EmployeeCategory::cost_per_millisecond`]
    #[must_use]
    pub fn salary(&self) -> u64 {
        self.salary
    }

    /// Computes the cost in dollars for each millisecond of time.
    ///
    /// ## Example
    /// ```
    /// use meeting_cost_tracker::EmployeeCategory;
    /// let cat = EmployeeCategory::new("Analyst", 90_000).unwrap();
    /// let ms = cat.cost_per_millisecond();
    /// assert!(ms > 0.0);
    /// ```
    ///
    /// # Arguments
    ///
    /// * None
    ///
    /// # Returns
    ///
    /// The cost per millisecond in dollars.
    ///
    /// # See Also
    /// * [`EmployeeCategory::salary`]
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn cost_per_millisecond(&self) -> f64 {
        self.salary as f64 / MILLIS_PER_WORK_YEAR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_validates_input() {
        assert!(EmployeeCategory::new("", 100).is_err());
        assert!(EmployeeCategory::new("dev", 0).is_err());
        let cat = EmployeeCategory::new("dev", 1).unwrap();
        assert_eq!(cat.title(), "dev");
        assert_eq!(cat.salary(), 1);
    }

    #[test]
    fn accessors_work() {
        let cat = EmployeeCategory::new("manager", 10_000).unwrap();
        assert_eq!(cat.title(), "manager");
        assert_eq!(cat.salary(), 10_000);
    }

    #[test]
    fn cost_per_millisecond_calculates_float() {
        let cat = EmployeeCategory::new("engineer", 720_000_000).unwrap();
        let cost = cat.cost_per_millisecond();
        assert!((cost - 0.1).abs() < f64::EPSILON);

        let small = EmployeeCategory::new("low", 100_000).unwrap();
        let cost = small.cost_per_millisecond();
        assert!(cost > 0.0);
    }
}
