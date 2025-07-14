use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::model::EmployeeCategory;
use thiserror::Error;

/// Errors that may occur during loading or saving categories.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("TOML serialization error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("TOML write error: {0}")]
    TomlWrite(#[from] toml::ser::Error),
}

/// Internal helper struct used for serializing and deserializing the collection
/// of [`EmployeeCategory`] values.
///
/// The wrapper is not exposed publicly but simplifies the structure of the TOML
/// file on disk.
#[derive(serde::Serialize, serde::Deserialize)]
struct CategoryWrapper {
    categories: Vec<EmployeeCategory>,
}

/// Represents a saved attendee entry of a specific title and count.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AttendeeInfo {
    /// Title of the attendee category.
    pub title: String,
    /// Number of attendees in this category.
    pub count: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AttendeesWrapper {
    attendees: Vec<AttendeeInfo>,
}

/// Loads employee categories from a TOML file.
///
/// If the file does not exist an empty collection is returned.
///
/// ## Example
/// ```
/// use std::path::Path;
/// use meeting_cost_tracker::load_categories;
/// let categories = load_categories(Path::new("categories.toml")).unwrap();
/// ```
///
/// # Arguments
///
/// * `path` - Path to the TOML file.
///
/// # Returns
///
/// A collection of [`EmployeeCategory`] values
///
/// # Errors
///
/// Returns a [`StorageError`] if the file cannot be read or if the contents
/// fail to parse as TOML.
///
/// # See Also
/// * [`save_categories`]
///
/// # Panics
///
/// This function does not panic.
pub fn load_categories(path: &Path) -> Result<Vec<EmployeeCategory>, StorageError> {
    if !path.exists() {
        return Ok(vec![]);
    }
    let data = fs::read_to_string(path)?;
    let wrapper: CategoryWrapper = toml::from_str(&data)?;
    Ok(wrapper.categories)
}

/// Persists employee categories to a TOML file, overwriting any existing content.
///
/// ## Example
/// ```
/// use std::path::Path;
/// use meeting_cost_tracker::{save_categories, EmployeeCategory};
/// let categories = vec![EmployeeCategory::new("Engineer", 100_000).unwrap()];
/// save_categories(Path::new("categories.toml"), &categories).unwrap();
/// ```
///
/// # Arguments
///
/// * `path` - Destination TOML file.
/// * `categories` - Employee categories to store.
///
/// # Returns
///
/// Result indicating success or failure.
///
/// # Errors
///
/// Returns a [`StorageError`] if the file cannot be created or written, or if
/// serialization of the categories fails.
///
/// # See Also
/// * [`load_categories`]
///
/// # Panics
///
/// This function does not panic.
pub fn save_categories<P: AsRef<Path>>(
    path: P,
    categories: &[EmployeeCategory],
) -> Result<(), StorageError> {
    let wrapper = CategoryWrapper {
        categories: categories.to_vec(),
    };
    let toml = toml::to_string_pretty(&wrapper)?;
    let mut file = fs::File::create(path)?;
    file.write_all(toml.as_bytes())?;
    Ok(())
}

/// Loads meeting attendees from a TOML file.
/// Missing files return an empty collection.
pub fn load_attendees(path: &Path) -> Result<Vec<AttendeeInfo>, StorageError> {
    if !path.exists() {
        return Ok(vec![]);
    }
    let data = fs::read_to_string(path)?;
    let wrapper: AttendeesWrapper = toml::from_str(&data)?;
    Ok(wrapper.attendees)
}

/// Persists meeting attendees to a TOML file.
pub fn save_attendees<P: AsRef<Path>>(
    path: P,
    attendees: &[AttendeeInfo],
) -> Result<(), StorageError> {
    let wrapper = AttendeesWrapper {
        attendees: attendees.to_vec(),
    };
    let toml = toml::to_string_pretty(&wrapper)?;
    let mut file = fs::File::create(path)?;
    file.write_all(toml.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn load_nonexistent_returns_empty() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();
        drop(tmp); // remove file so it doesn't exist
        let cats = load_categories(&path).unwrap();
        assert!(cats.is_empty());
    }

    #[test]
    fn save_and_load_round_trip() {
        let tmp = NamedTempFile::new().unwrap();
        let cats = vec![EmployeeCategory::new("A", 1).unwrap()];
        save_categories(tmp.path(), &cats).unwrap();
        let loaded = load_categories(tmp.path()).unwrap();
        assert_eq!(cats, loaded);
    }

    #[test]
    fn load_invalid_toml_errors() {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(b"invalid_toml").unwrap();
        let res = load_categories(tmp.path());
        assert!(res.is_err());
    }

    #[test]
    fn attendees_round_trip() {
        let tmp = NamedTempFile::new().unwrap();
        let attendees = vec![AttendeeInfo {
            title: "Dev".into(),
            count: 3,
        }];
        save_attendees(tmp.path(), &attendees).unwrap();
        let loaded = load_attendees(tmp.path()).unwrap();
        assert_eq!(attendees, loaded);
    }
}
