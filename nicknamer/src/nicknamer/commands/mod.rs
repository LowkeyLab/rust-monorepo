use crate::nicknamer::discord;
use poise::serenity_prelude as serenity;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

pub mod reveal;

type Reply = String;

#[derive(Debug, PartialEq, Default)]
pub struct User {
    pub id: u64,
    pub display_name: String,
    pub real_name: String,
}

impl User {
    pub fn from_discord_server_member(
        discord_member: &discord::ServerMember,
        real_name: String,
    ) -> Self {
        Self {
            id: discord_member.id,
            display_name: discord_member
                .nick_name
                .clone()
                .unwrap_or_else(|| discord_member.user_name.clone()),
            real_name,
        }
    }
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}' is {}", self.display_name, self.real_name)
    }
}

#[derive(Debug, PartialEq)]
pub struct RealNames {
    pub(crate) users: HashMap<u64, User>,
}

impl Display for RealNames {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.users
                .iter()
                .map(|(_, user)| format!("{}", user))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

#[allow(dead_code)]
pub fn nick(_user_id: serenity::UserId) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nicknamer::discord::ServerMember;

    #[test]
    fn test_from_discord_server_member_with_nickname() {
        // Arrange
        let member = ServerMember {
            id: 12345,
            nick_name: Some("NickName".to_string()),
            user_name: "UserName".to_string(),
        };
        let real_name = "Real Name".to_string();

        // Act
        let user = User::from_discord_server_member(&member, real_name);

        // Assert
        assert_eq!(user.id, 12345);
        assert_eq!(user.display_name, "NickName");
        assert_eq!(user.real_name, "Real Name");
    }

    #[test]
    fn test_from_discord_server_member_without_nickname() {
        // Arrange
        let member = ServerMember {
            id: 67890,
            nick_name: None,
            user_name: "UserName".to_string(),
        };
        let real_name = "Real Name".to_string();

        // Act
        let user = User::from_discord_server_member(&member, real_name);

        // Assert
        assert_eq!(user.id, 67890);
        assert_eq!(user.display_name, "UserName"); // Should fall back to username
        assert_eq!(user.real_name, "Real Name");
    }

    #[test]
    fn test_from_discord_server_member_empty_nickname() {
        // Arrange
        let member = ServerMember {
            id: 13579,
            nick_name: Some("".to_string()), // Empty nickname
            user_name: "UserName".to_string(),
        };
        let real_name = "Real Name".to_string();

        // Act
        let user = User::from_discord_server_member(&member, real_name);

        // Assert
        assert_eq!(user.id, 13579);
        assert_eq!(user.display_name, ""); // Should use the empty nickname
        assert_eq!(user.real_name, "Real Name");
    }

    #[test]
    fn test_from_discord_server_member_with_matching_names() {
        // Arrange - when nickname matches username
        let member = ServerMember {
            id: 24680,
            nick_name: Some("SameName".to_string()),
            user_name: "SameName".to_string(),
        };
        let real_name = "Real Name".to_string();

        // Act
        let user = User::from_discord_server_member(&member, real_name);

        // Assert
        assert_eq!(user.id, 24680);
        assert_eq!(user.display_name, "SameName");
        assert_eq!(user.real_name, "Real Name");
    }
}
