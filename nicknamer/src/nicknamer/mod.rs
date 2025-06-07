pub(crate) mod commands;
mod config;

pub(crate) mod connectors;

use commands::Error;
use commands::names::NamesRepository;
use commands::reveal::{Revealer, RevealerImpl};
use connectors::discord::DiscordConnector;

trait Nicknamer {
    async fn reveal_all(&self) -> Result<(), Error>;
}

struct NicknamerImpl<'a, REPO: NamesRepository, DISCORD: DiscordConnector> {
    names_repository: &'a REPO,
    discord_connector: &'a DISCORD,
}

impl<'a, REPO: NamesRepository, DISCORD: DiscordConnector> NicknamerImpl<'a, REPO, DISCORD> {
    fn new(names_repository: &'a REPO, discord_connector: &'a DISCORD) -> Self {
        Self {
            names_repository,
            discord_connector,
        }
    }
}

impl<'a, REPO: NamesRepository, DISCORD: DiscordConnector> Nicknamer
    for NicknamerImpl<'a, REPO, DISCORD>
{
    async fn reveal_all(&self) -> Result<(), Error> {
        let revealer = RevealerImpl::new(self.names_repository, self.discord_connector);
        revealer.reveal_all().await
    }
}
