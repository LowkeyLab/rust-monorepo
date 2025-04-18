use serde::{Deserialize, Serialize};
use std::error::Error;

// Include the YAML content at compile time
const REAL_NAMES_YAML: &str = include_str!("real_names.yml");

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RealNames {
    pub(crate) names: std::collections::HashMap<u64, String>,
}

impl RealNames {
    /// Creates a new RealNames struct by parsing the embedded YAML content
    ///
    /// # Returns
    ///
    /// * `Result<RealNames, Box<dyn Error>>` - A new RealNames struct or an error
    pub fn from_embedded_yaml() -> Result<Self, Box<dyn Error>> {
        Self::from_yaml(REAL_NAMES_YAML)
    }

    /// Parses a YAML string into a RealNames struct
    ///
    /// # Arguments
    ///
    /// * `yaml_string` - A string containing YAML formatted data with the names mapping
    ///
    /// # Returns
    ///
    /// * `Result<RealNames, Box<dyn Error>>` - A RealNames struct on success, or an error if parsing fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The YAML syntax is invalid
    /// * The YAML structure doesn't match the RealNames struct format
    ///
    /// # Example
    ///
    /// ```
    /// let yaml_data = r#"
    /// names:
    ///   123456789: Alice
    ///   987654321: Bob
    /// "#;
    ///
    /// let real_names = RealNames::from_yaml(yaml_data).unwrap();
    /// ```
    pub fn from_yaml(yaml_string: &str) -> Result<Self, Box<dyn Error>> {
        let real_names: RealNames = serde_yml::from_str(yaml_string)?;
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

        // Deserialize the YAML string
        let deserialized: RealNames = serde_yml::from_str(yaml_data).unwrap();

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
