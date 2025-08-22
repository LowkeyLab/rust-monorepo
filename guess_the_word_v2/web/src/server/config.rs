use serde::Deserialize;

/// Server configuration loaded from environment variables
#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    /// Database connection URL
    pub database_url: String,
}

impl ServerConfig {
    /// Load configuration from environment variables
    pub fn load() -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            .add_source(config::Environment::default())
            .build()?;

        settings.try_deserialize()
    }
}
