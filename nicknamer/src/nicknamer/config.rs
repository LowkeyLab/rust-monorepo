use serde::Deserialize;

pub const REVEAL_INSULT: &str = "ya dingus";
pub const CODE_MONKEYS_ROLE_NAME: &str = "Code Monkeys";

#[derive(Debug, Deserialize)]
pub struct Config {
    pub nicknamer: NicknamerConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RevealConfig {
    pub insult: String,
    pub role_to_mention: String,
}

#[derive(Clone, Debug, Deserialize)]
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
