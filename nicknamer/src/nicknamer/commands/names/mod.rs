use log::info;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to parse YAML")]
    SerdeYamlError(#[from] serde_yml::Error),
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Names {
    pub(crate) names: std::collections::HashMap<u64, String>,
}

pub trait NamesRepository {
    async fn load_real_names(&self) -> Result<Names, Error>;
}

pub struct EmbeddedNamesRepository {
    embedded_names: &'static str,
}

impl EmbeddedNamesRepository {
    pub(crate) fn new() -> Self {
        Self {
            embedded_names: include_str!("real_names.yml"),
        }
    }
}

impl NamesRepository for EmbeddedNamesRepository {
    async fn load_real_names(&self) -> Result<Names, Error> {
        let names: Names = serde_yml::from_str(self.embedded_names)?;
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
