use crate::nicknamer::commands::names::{Names, NamesRepository};
use crate::nicknamer::commands::{Error, Reply, User};
use crate::nicknamer::config;
use crate::nicknamer::connectors::discord::server_member::ServerMember;
use crate::nicknamer::connectors::discord::{DiscordConnector, Role};
use log::info;

pub trait Revealer {
    async fn reveal_all(&self) -> Result<(), Error>;
    async fn reveal_member(&self, member: &ServerMember) -> Result<(), Error>;
}
pub struct RevealerImpl<'a, REPO: NamesRepository, DISCORD: DiscordConnector> {
    names_repository: &'a REPO,
    discord_connector: &'a DISCORD,
}

impl<'a, NAMES: NamesRepository, DISCORD: DiscordConnector> RevealerImpl<'a, NAMES, DISCORD> {
    pub fn new(names_repository: &'a NAMES, discord_connector: &'a DISCORD) -> Self {
        Self {
            names_repository,
            discord_connector,
        }
    }
}

impl<'a, REPO: NamesRepository, DISCORD: DiscordConnector> Revealer
    for RevealerImpl<'a, REPO, DISCORD>
{
    async fn reveal_all(&self) -> Result<(), Error> {
        info!("Revealing real names for current channel members ...");
        let members = self
            .discord_connector
            .get_members_of_current_channel()
            .await?;
        let real_names = self.names_repository.load_real_names().await?;
        self.reveal_all_members(&members, &real_names).await?;
        Ok(())
    }

    async fn reveal_member(&self, member: &ServerMember) -> Result<(), Error> {
        info!("Revealing real name for {}", member.user_name);
        if member.is_bot {
            self.reveal_bot_member(member).await?;
        } else {
            self.reveal_human_member(member).await?;
        }
        Ok(())
    }
}

impl<'a, REPO: NamesRepository, DISCORD: DiscordConnector> RevealerImpl<'a, REPO, DISCORD> {
    async fn reveal_human_member(&self, member: &ServerMember) -> Result<(), Error> {
        let names = self.names_repository.load_real_names().await?;
        let reply = reveal_member(member, &names);
        self.discord_connector.send_reply(&reply).await?;
        Ok(())
    }
}

impl<'a, REPO: NamesRepository, DISCORD: DiscordConnector> RevealerImpl<'a, REPO, DISCORD> {
    async fn reveal_bot_member(&self, member: &ServerMember) -> Result<(), Error> {
        let name_to_show = match &member.nick_name {
            Some(nick_name) => nick_name,
            None => &member.user_name,
        };
        let reply = format!("{} is a bot, {}!", name_to_show, config::REVEAL_INSULT);
        self.discord_connector.send_reply(&reply).await?;
        Ok(())
    }
}

impl<'a, REPO: NamesRepository, DISCORD: DiscordConnector> RevealerImpl<'a, REPO, DISCORD> {
    async fn reveal_all_members(
        &self,
        members: &[ServerMember],
        real_names: &Names,
    ) -> Result<(), Error> {
        self.reveal_users_with_real_name(members, real_names)
            .await?;
        self.reveal_users_without_real_name(members, real_names)
            .await?;
        Ok(())
    }

    async fn reveal_users_with_real_name(
        &self,
        members: &[ServerMember],
        real_names: &Names,
    ) -> Result<(), Error> {
        let users: Vec<User> = get_users_with_real_names(&members, &real_names);
        info!("Found {} users with real names", users.len());
        if users.is_empty() {
            return Ok(());
        }
        let reply_for_users_with_real_name = create_reply_for_users_with_real_names(&users);
        self.discord_connector
            .send_reply(&reply_for_users_with_real_name)
            .await?;
        Ok(())
    }

    async fn reveal_users_without_real_name(
        &self,
        members: &[ServerMember],
        real_names: &Names,
    ) -> Result<(), Error> {
        let users: Vec<User> = get_users_without_real_names(members, real_names);
        info!("Found {} users without real names", users.len());
        if users.is_empty() {
            return Ok(());
        }
        let role_to_mention = self
            .discord_connector
            .get_role_by_name(config::CODE_MONKEYS_ROLE_NAME)
            .await?;
        let reply = create_reply_for_users_without_real_names(&users, &*role_to_mention);
        if reply.is_empty() {
            return Ok(());
        }
        self.discord_connector.send_reply(&reply).await?;
        Ok(())
    }
}

fn reveal_member(server_member: &ServerMember, real_names: &Names) -> Reply {
    let user_id = server_member.id;
    let mut user: User = server_member.into();
    let real_name = real_names.names.get(&user_id).cloned();
    user.real_name = real_name;
    user.to_string()
}

fn get_users_with_real_names(members: &[ServerMember], real_names: &Names) -> Vec<User> {
    members
        .iter()
        .filter_map(|member| {
            // Only include users with real names in our database
            let Some(real_name) = real_names.names.get(&member.id) else {
                return None;
            };
            let mut user: User = member.into();
            user.real_name = Some(real_name.clone());
            Some(user)
        })
        .collect()
}

fn get_users_without_real_names(members: &[ServerMember], real_names: &Names) -> Vec<User> {
    members
        .iter()
        .filter_map(|member| {
            let None = real_names.names.get(&member.id) else {
                return None;
            };
            if member.is_bot {
                return None;
            }
            let user: User = member.into();
            Some(user)
        })
        .collect()
}

fn create_reply_for_users_with_real_names(users: &[User]) -> Reply {
    assert!(
        !users.is_empty(),
        "You can't create a reply for an empty list of users"
    );
    let reply = users
        .into_iter()
        .map(|user| user.to_string())
        .collect::<Vec<String>>();

    format!(
        "Here are people's real names, {}:
\t{}",
        config::REVEAL_INSULT,
        reply.join("\n\t")
    )
}

fn create_reply_for_users_without_real_names(users: &[User], mention: &dyn Role) -> Reply {
    assert!(
        !users.is_empty(),
        "You can't create a reply for an empty list of users"
    );
    let reply = users
        .into_iter()
        .map(|user| user.to_string())
        .collect::<Vec<String>>();

    format!(
        "Hey {}, these members are unrecognized:
\t{}
One of y'all should improve real name management and/or add them to the config",
        mention.mention(),
        reply.join("\n\t")
    )
}
