use crate::nicknamer::connectors::discord;
use poise::serenity_prelude as serenity;
use std::fmt::{Display, Formatter};

pub(crate) mod names;
pub mod reveal;

pub(crate) type Reply = String;

#[derive(Debug, PartialEq, Default)]
pub struct User {
    pub id: u64,
    pub display_name: String,
    pub real_name: Option<String>,
}
impl From<discord::ServerMember> for User {
    fn from(discord_member: discord::ServerMember) -> Self {
        Self {
            id: discord_member.id,
            display_name: discord_member
                .nick_name
                .clone()
                .unwrap_or_else(|| discord_member.user_name.clone()),
            real_name: None,
        }
    }
}

impl From<&discord::ServerMember> for User {
    fn from(discord_member: &discord::ServerMember) -> Self {
        Self {
            id: discord_member.id,
            display_name: discord_member
                .nick_name
                .clone()
                .unwrap_or_else(|| discord_member.user_name.clone()),
            real_name: None,
        }
    }
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(real_name) = &self.real_name {
            write!(f, "'{}' is {}", self.display_name, real_name)
        } else {
            write!(f, "'{}' has no real name available", self.display_name)
        }
    }
}

#[allow(dead_code)]
pub fn nick(_user_id: serenity::UserId) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nicknamer::connectors::discord::ServerMember;

    #[test]
    fn test_from_discord_server_member_with_nickname() {
        // Arrange
        let member = ServerMember {
            id: 12345,
            nick_name: Some("NickName".to_string()),
            user_name: "UserName".to_string(),
        };
        let real_name = Some("Real Name".to_string());

        // Act
        let mut user = User::from(member);
        user.real_name = real_name;

        // Assert
        assert_eq!(user.id, 12345);
        assert_eq!(user.display_name, "NickName");
        assert_eq!(user.real_name, Some("Real Name".into()));
    }

    #[test]
    fn test_from_discord_server_member_without_nickname() {
        // Arrange
        let member = ServerMember {
            id: 67890,
            nick_name: None,
            user_name: "UserName".to_string(),
        };
        let real_name = Some("Real Name".to_string());

        // Act
        let mut user = User::from(member);
        user.real_name = real_name;

        // Assert
        assert_eq!(user.id, 67890);
        assert_eq!(user.display_name, "UserName"); // Should fall back to username
        assert_eq!(user.real_name, Some("Real Name".into()));
    }

    #[test]
    fn test_from_discord_server_member_empty_nickname() {
        // Arrange
        let member = ServerMember {
            id: 13579,
            nick_name: Some("".to_string()), // Empty nickname
            user_name: "UserName".to_string(),
        };
        let real_name = Some("Real Name".to_string());

        // Act
        let mut user = User::from(member);
        user.real_name = real_name;

        // Assert
        assert_eq!(user.id, 13579);
        assert_eq!(user.display_name, ""); // Should use the empty nickname
        assert_eq!(user.real_name, Some("Real Name".into()));
    }

    #[test]
    fn test_from_discord_server_member_with_matching_names() {
        // Arrange - when nickname matches username
        let member = ServerMember {
            id: 24680,
            nick_name: Some("SameName".to_string()),
            user_name: "SameName".to_string(),
        };
        let real_name = Some("Real Name".to_string());

        // Act
        let mut user = User::from(member);
        user.real_name = real_name;

        // Assert
        assert_eq!(user.id, 24680);
        assert_eq!(user.display_name, "SameName");
        assert_eq!(user.real_name, Some("Real Name".into()));
    }

    #[test]
    fn test_user_display_with_real_name() {
        // Arrange
        let user = User {
            id: 12345,
            display_name: "DisplayName".to_string(),
            real_name: Some("RealName".to_string()),
        };

        // Act
        let display_string = format!("{}", user);

        // Assert
        assert_eq!(display_string, "'DisplayName' is RealName");
    }

    #[test]
    fn test_user_display_without_real_name() {
        // Arrange
        let user = User {
            id: 12345,
            display_name: "DisplayName".to_string(),
            real_name: None,
        };

        // Act
        let display_string = format!("{}", user);

        // Assert
        assert_eq!(display_string, "'DisplayName' has no real name available");
    }

    #[test]
    fn test_user_display_with_empty_display_name() {
        // Arrange
        let user = User {
            id: 12345,
            display_name: "".to_string(),
            real_name: Some("RealName".to_string()),
        };

        // Act
        let display_string = format!("{}", user);

        // Assert
        assert_eq!(display_string, "'' is RealName");
    }

    #[test]
    fn test_user_display_with_special_characters() {
        // Arrange
        let user = User {
            id: 12345,
            display_name: "Name\"With'Quotes".to_string(),
            real_name: Some("Real\"Name'With'Quotes".to_string()),
        };

        // Act
        let display_string = format!("{}", user);

        // Assert
        assert_eq!(
            display_string,
            "'Name\"With'Quotes' is Real\"Name'With'Quotes"
        );
    }

    #[test]
    fn test_user_display_with_empty_real_name() {
        // Arrange - edge case with empty string (not None) for real_name
        let user = User {
            id: 12345,
            display_name: "DisplayName".to_string(),
            real_name: Some("".to_string()),
        };

        // Act
        let display_string = format!("{}", user);

        // Assert
        assert_eq!(display_string, "'DisplayName' is ");
    }
}
