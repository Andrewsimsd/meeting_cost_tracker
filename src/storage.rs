use std::fs;
use std::path::Path;
use std::io::{self, Write};

use crate::model::EmployeeCategory;
use thiserror::Error;

/// Filename for persistent employee category storage.
pub const STORAGE_FILE: &str = "categories.toml";

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

#[derive(serde::Serialize, serde::Deserialize)]
struct CategoryWrapper {
    categories: Vec<EmployeeCategory>,
}

/// Loads categories from a TOML file. Returns an empty Vec if the file doesn't exist.
pub fn load_categories(path: &Path) -> Result<Vec<EmployeeCategory>, StorageError> {
    if !path.exists() {
        return Ok(vec![]);
    }
    let data = fs::read_to_string(path)?;
    let wrapper: CategoryWrapper = toml::from_str(&data)?;
    Ok(wrapper.categories)
}

/// Saves categories to a TOML file, overwriting any existing content.
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
