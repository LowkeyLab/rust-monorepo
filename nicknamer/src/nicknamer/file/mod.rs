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
        let real_names: RealNames = serde_yml::from_str(REAL_NAMES_YAML).or(Err(FileError {
            description: "Failed to read embedded YAML file".to_string(),
        }))?;
        Ok(real_names)
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
        let deserialized = serde_yml::from_str::<RealNames>(yaml_data).unwrap();

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
