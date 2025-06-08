use serde::Deserialize;

pub const REVEAL_INSULT: &str = "ya dingus";
pub const CODE_MONKEYS_ROLE_NAME: &str = "Code Monkeys";

#[derive(Debug, Deserialize)]
pub struct Config {
    nicknamer: NicknamerConfig,
}

#[derive(Debug, Deserialize)]
pub struct RevealConfig {
    pub insult: String,
    pub role_to_mention: String,
}

#[derive(Debug, Deserialize)]
pub struct NicknamerConfig {
    reveal: RevealConfig,
}

impl Config {
    fn new() -> anyhow::Result<Self> {
        let s = config::Config::builder()
            .add_source(config::File::with_name("nicknamer/config"))
            .build()?;

        Ok(s.try_deserialize()?)
    }
}
