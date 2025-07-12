use serde::{Deserialize, Serialize};
use thiserror::Error;

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmployeeCategory {
    title: String,
    salary: f64,
}

impl EmployeeCategory {
    /// Creates a new [`EmployeeCategory`] with validation.
    ///
    /// # Errors
    ///
    /// Returns [`EmployeeCategoryError`] if title is empty or salary is non-positive.
    pub fn new<T: Into<String>>(title: T, salary: f64) -> Result<Self, EmployeeCategoryError> {
        let title = title.into();
        if title.trim().is_empty() {
            return Err(EmployeeCategoryError::EmptyTitle);
        }
        if salary <= 0.0 {
            return Err(EmployeeCategoryError::InvalidSalary);
        }
        Ok(Self { title, salary })
    }

    /// Returns the title of the employee category.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the annual salary.
    #[must_use]
    pub fn salary(&self) -> f64 {
        self.salary
    }

    /// Computes the cost per millisecond.
    #[must_use]
    pub fn cost_per_millisecond(&self) -> f64 {
        self.salary / (365.25 * 24.0 * 60.0 * 60.0 * 1000.0)
    }
}