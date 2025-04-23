use crate::nicknamer::connectors::discord;
use poise::serenity_prelude as serenity;
use std::fmt::{Display, Formatter};

pub(crate) mod names;
pub mod reveal;

pub(crate) type Reply = String;

#[derive(Debug, PartialEq, Default)]
pub struct User {
    pub id: u64,
    pub nick_name: String,
    pub real_name: Option<String>,
}
impl From<discord::ServerMember> for User {
    fn from(discord_member: discord::ServerMember) -> Self {
        Self {
            id: discord_member.id,
            nick_name: discord_member
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
            nick_name: discord_member
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
            write!(f, "'{}' is {}", self.nick_name, real_name)
        } else {
            write!(f, "'{}' has no real name available", self.nick_name)
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
    fn can_convert_server_member_with_nickname_to_user() {
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
        assert_eq!(user.nick_name, "NickName");
        assert_eq!(user.real_name, Some("Real Name".into()));
    }

    #[test]
    fn can_convert_server_member_without_nickname_using_username_fallback() {
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
        assert_eq!(user.nick_name, "UserName"); // Should fall back to username
        assert_eq!(user.real_name, Some("Real Name".into()));
    }

    #[test]
    fn can_handle_empty_nickname_when_converting_server_member() {
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
        assert_eq!(user.nick_name, ""); // Should use the empty nickname
        assert_eq!(user.real_name, Some("Real Name".into()));
    }

    #[test]
    fn can_convert_server_member_when_nickname_matches_username() {
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
        assert_eq!(user.nick_name, "SameName");
        assert_eq!(user.real_name, Some("Real Name".into()));
    }

    #[test]
    fn can_format_user_with_real_name_for_display() {
        // Arrange
        let user = User {
            id: 12345,
            nick_name: "DisplayName".to_string(),
            real_name: Some("RealName".to_string()),
        };

        // Act
        let display_string = format!("{}", user);

        // Assert
        assert_eq!(display_string, "'DisplayName' is RealName");
    }

    #[test]
    fn can_format_user_without_real_name_for_display() {
        // Arrange
        let user = User {
            id: 12345,
            nick_name: "DisplayName".to_string(),
            real_name: None,
        };

        // Act
        let display_string = format!("{}", user);

        // Assert
        assert_eq!(display_string, "'DisplayName' has no real name available");
    }

    #[test]
    fn can_format_user_with_empty_display_name() {
        // Arrange
        let user = User {
            id: 12345,
            nick_name: "".to_string(),
            real_name: Some("RealName".to_string()),
        };

        // Act
        let display_string = format!("{}", user);

        // Assert
        assert_eq!(display_string, "'' is RealName");
    }

    #[test]
    fn can_handle_special_characters_when_formatting_user() {
        // Arrange
        let user = User {
            id: 12345,
            nick_name: "Name\"With'Quotes".to_string(),
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
    fn can_format_user_with_empty_real_name_string() {
        // Arrange - edge case with empty string (not None) for real_name
        let user = User {
            id: 12345,
            nick_name: "DisplayName".to_string(),
            real_name: Some("".to_string()),
        };

        // Act
        let display_string = format!("{}", user);

        // Assert
        assert_eq!(display_string, "'DisplayName' is ");
    }
}
