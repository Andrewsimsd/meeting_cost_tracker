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
/// A collection of [`EmployeeCategory`] values.

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
/// let categories = vec![EmployeeCategory::new("Engineer", 100_000.0).unwrap()];
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
