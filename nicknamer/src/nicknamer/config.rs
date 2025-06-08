use serde::{Deserialize, Serialize};

pub const REVEAL_INSULT: &str = "ya dingus";
pub const CODE_MONKEYS_ROLE_NAME: &str = "Code Monkeys";

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub nicknamer: NicknamerConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RevealConfig {
    pub insult: String,
    pub role_to_mention: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NicknamerConfig {
    pub(crate) reveal: RevealConfig,
}

impl NicknamerConfig {
    pub fn reveal(&self) -> &RevealConfig {
        &self.reveal
    }
}

impl Config {
    pub(crate) fn new() -> anyhow::Result<Self> {
        let s = config::Config::builder()
            .add_source(config::File::with_name("nicknamer/config"))
            .build()?;

        Ok(s.try_deserialize()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod deser_tests {
        use super::*;

        #[test]
        fn test_config_deserialize_from_toml() {
            // Arrange
            let toml_str = r#"
                [nicknamer]
                [nicknamer.reveal]
                insult = "test insult"
                role_to_mention = "test role"
            "#;

            // Act
            let config: Config = toml::from_str(toml_str).unwrap();

            // Assert
            assert_eq!(config.nicknamer.reveal.insult, "test insult");
            assert_eq!(config.nicknamer.reveal.role_to_mention, "test role");
        }

        #[test]
        fn test_config_deserialize_empty_fields() {
            // Arrange
            let toml_str = r#"
                [nicknamer]
                [nicknamer.reveal]
                insult = ""
                role_to_mention = ""
            "#;

            // Act
            let config: Config = toml::from_str(toml_str).unwrap();

            // Assert
            assert_eq!(config.nicknamer.reveal.insult, "");
            assert_eq!(config.nicknamer.reveal.role_to_mention, "");
        }

        #[test]
        fn test_config_deserialize_special_characters() {
            // Arrange
            let toml_str = r#"
                [nicknamer]
                [nicknamer.reveal]
                insult = "Special chars: !@#$%^&*()"
                role_to_mention = "More special: 😊🚀👍"
            "#;

            // Act
            let config: Config = toml::from_str(toml_str).unwrap();

            // Assert
            assert_eq!(config.nicknamer.reveal.insult, "Special chars: !@#$%^&*()");
            assert_eq!(
                config.nicknamer.reveal.role_to_mention,
                "More special: 😊🚀👍"
            );
        }
    }

    #[test]
    fn test_config_serialize_to_toml() {
        // Arrange
        let config = Config {
            nicknamer: NicknamerConfig {
                reveal: RevealConfig {
                    insult: "test insult".to_string(),
                    role_to_mention: "test role".to_string(),
                },
            },
        };

        // Act
        let toml_str = toml::to_string(&config).unwrap();

        // Assert
        assert!(toml_str.contains("insult = \"test insult\""));
        assert!(toml_str.contains("role_to_mention = \"test role\""));
    }

    #[test]
    fn test_config_roundtrip() {
        // Arrange
        let original_config = Config {
            nicknamer: NicknamerConfig {
                reveal: RevealConfig {
                    insult: "roundtrip insult".to_string(),
                    role_to_mention: "roundtrip role".to_string(),
                },
            },
        };

        // Act: Serialize to TOML
        let toml_str = toml::to_string(&original_config).unwrap();

        // Act: Deserialize back to Config
        let deserialized_config: Config = toml::from_str(&toml_str).unwrap();

        // Assert
        assert_eq!(
            deserialized_config.nicknamer.reveal.insult,
            "roundtrip insult"
        );
        assert_eq!(
            deserialized_config.nicknamer.reveal.role_to_mention,
            "roundtrip role"
        );
    }
}
