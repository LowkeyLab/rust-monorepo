use crate::nicknamer::commands::Error;

pub trait Nick {
    async fn nick(&self, nick: &str) -> Result<(), Error>;
}
