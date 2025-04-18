use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
// Include the YAML content at compile time
const REAL_NAMES_YAML: &str = include_str!("real_names.yml");

#[derive(Debug)]
pub struct FileError {
    description: String,
}

impl Display for FileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl std::error::Error for FileError {}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RealNames {
    pub(crate) names: std::collections::HashMap<u64, String>,
}

impl RealNames {
    /// Creates a new RealNames struct by parsing the embedded YAML content
    pub fn from_embedded_yaml() -> Result<Self, FileError> {
        Self::from_yaml(REAL_NAMES_YAML)
    }

    /// Parse a YAML string into a RealNames struct
    ///
    /// # Arguments
    ///
    /// * `yaml_string` - A string containing YAML formatted data with the names mapping
    ///
    /// # Returns
    ///
    /// * `Result<Self, FileError>` - A RealNames struct on success, or an error if parsing fails
    pub fn from_yaml(yaml_string: &str) -> Result<Self, FileError> {
        serde_yml::from_str(yaml_string).or(Err(FileError {
            description: "Failed to parse YAML content".to_string(),
        }))
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
        let deserialized = RealNames::from_yaml(yaml_data).unwrap();

        // Create the expected RealNames object for comparison
        let mut expected = RealNames {
            names: HashMap::new(),
        };
        expected.names.insert(123456789, "Alice".to_string());
        expected.names.insert(987654321, "Bob".to_string());

        // Assert that deserialization produced the expected object
        assert_eq!(deserialized, expected);
    }
}
