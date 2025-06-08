//! Names module for handling user real name data.
//!
//! This module provides functionality for loading, storing, and accessing mappings
//! between Discord user IDs and their real names. It includes:
//! - Data structures for representing collections of user real names
//! - A repository trait for loading name data
//! - An implementation that loads names from an embedded YAML file
//!
//! The module supports deserializing name data from YAML format and provides
//! error handling for failed loading operations.

use crate::nicknamer::names::Error::CannotLoadNames;
use async_trait::async_trait;
use log::info;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during name operations.
///
/// These errors represent failures that might occur when
/// loading or processing user real name data.
#[derive(Error, Debug)]
pub enum Error {
    /// Indicates a failure to load the names data, typically from YAML parsing
    #[error("Failed to load names")]
    CannotLoadNames,
}

/// Collection of user real names indexed by Discord user IDs.
///
/// This structure stores a mapping of Discord user IDs to their corresponding
/// real names, allowing for quick lookups. It is designed to be serialized
/// to and deserialized from YAML format.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Names {
    /// Mapping of Discord user IDs to real names
    pub(crate) names: std::collections::HashMap<u64, String>,
}

/// Trait defining operations for accessing user real name data.
///
/// Implementations of this trait provide mechanisms for loading
/// real name data from various sources.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait NamesRepository {
    /// Loads the real names collection from the repository.
    ///
    /// # Returns
    ///
    /// * `Result<Names, Error>` - The loaded names on success, or an error if loading fails
    async fn load_real_names(&self) -> Result<Names, Error>;
}

/// Repository implementation that loads names from an embedded YAML file.
///
/// This implementation includes the names data directly in the binary,
/// allowing for deployment without external data files.
pub struct EmbeddedNamesRepository {
    /// Contents of the embedded YAML file containing real names
    embedded_names: &'static str,
}

impl EmbeddedNamesRepository {
    /// Creates a new instance of the embedded names repository.
    ///
    /// This constructor includes the contents of the real_names.yml file
    /// directly in the binary at compile time.
    ///
    /// # Returns
    ///
    /// A new EmbeddedNamesRepository instance
    pub(crate) fn new() -> Self {
        Self {
            embedded_names: include_str!("real_names.yml"),
        }
    }
}

#[async_trait]
impl NamesRepository for EmbeddedNamesRepository {
    /// Loads real names from the embedded YAML data.
    ///
    /// Deserializes the embedded YAML string into a Names collection.
    /// Logs the number of names loaded for debugging purposes.
    ///
    /// # Returns
    ///
    /// * `Result<Names, Error>` - The loaded Names on success, or CannotLoadNames error on failure
    async fn load_real_names(&self) -> Result<Names, Error> {
        let names: Names = serde_yml::from_str(self.embedded_names).map_err(|_| CannotLoadNames)?;
        info!("Loaded {} names", names.names.len());
        Ok(names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_real_name_deser() {
        // Create a YAML string representing RealNames
        let yaml_data = r#"
names:
  123456789: Alice
  987654321: Bob
"#;

        // Deserialize the YAML string using from_yaml
        let deserialized: Names = serde_yml::from_str(yaml_data).unwrap();

        // Create the expected RealNames object for comparison
        let mut expected = Names {
            names: HashMap::new(),
        };
        expected.names.insert(123456789, "Alice".to_string());
        expected.names.insert(987654321, "Bob".to_string());

        // Assert that deserialization produced the expected object
        assert_eq!(deserialized, expected);
    }
}
