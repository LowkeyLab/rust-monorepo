use crate::nicknamer::connectors::discord;
use crate::nicknamer::connectors::discord::server_member;
use std::fmt::{Display, Formatter};
use thiserror::Error;

pub(crate) mod names;
pub mod reveal;

pub mod nick;

pub(crate) type Reply = String;

#[derive(Debug, PartialEq, Default)]
pub struct User {
    pub id: u64,
    pub user_name: String,
    pub nick_name: Option<String>,
    pub real_name: Option<String>,
}
impl From<server_member::ServerMember> for User {
    fn from(discord_member: server_member::ServerMember) -> Self {
        Self {
            id: discord_member.id,
            user_name: discord_member.user_name.clone(),
            nick_name: discord_member.nick_name.clone(),
            real_name: None,
        }
    }
}

impl From<&server_member::ServerMember> for User {
    fn from(discord_member: &server_member::ServerMember) -> Self {
        Self {
            id: discord_member.id,
            user_name: discord_member.user_name.clone(),
            nick_name: discord_member.nick_name.clone(),
            real_name: None,
        }
    }
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(real_name) = &self.real_name {
            self.display_user_with_real_name(f, real_name)
        } else {
            self.display_user_without_real_name(f)
        }
    }
}

impl User {
    fn display_user_with_real_name(
        &self,
        f: &mut Formatter,
        real_name: &String,
    ) -> std::fmt::Result {
        if let Some(nick_name) = &self.nick_name {
            write!(f, "'{}' is {}", nick_name, real_name)
        } else {
            write!(f, "'{}' is {}", self.user_name, real_name)
        }
    }

    fn display_user_without_real_name(&self, f: &mut Formatter) -> std::fmt::Result {
        if let Some(nick_name) = &self.nick_name {
            write!(f, "{} aka '{}'", self.user_name, nick_name)
        } else {
            write!(
                f,
                "{} has neither a nickname nor a real name",
                self.user_name
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nicknamer::connectors::discord::server_member::ServerMemberBuilder;

    #[test]
    fn can_convert_server_member_with_nickname_to_user() {
        // Arrange
        let member = ServerMemberBuilder::new()
            .id(12345)
            .nick_name("NickName")
            .user_name("UserName")
            .is_bot(false)
            .build();
        let real_name = Some("Real Name".to_string());

        // Act
        let mut user = User::from(member);
        user.real_name = real_name;

        // Assert
        assert_eq!(user.id, 12345);
        assert_eq!(user.nick_name, Some("NickName".to_string()));
        assert_eq!(user.real_name, Some("Real Name".into()));
    }

    #[test]
    fn can_convert_server_member_without_nickname_using_username_fallback() {
        // Arrange
        let member = ServerMemberBuilder::new()
            .id(67890)
            .user_name("UserName")
            .is_bot(false)
            .build();
        let real_name = Some("Real Name".to_string());

        // Act
        let mut user = User::from(member);
        user.real_name = real_name;

        // Assert
        assert_eq!(user.id, 67890);
        assert_eq!(user.nick_name, None); // No fallback behavior in the From implementation
        assert_eq!(user.real_name, Some("Real Name".into()));
    }

    #[test]
    fn can_handle_empty_nickname_when_converting_server_member() {
        // Arrange
        let member = ServerMemberBuilder::new()
            .id(13579)
            .nick_name("") // Empty nickname
            .user_name("UserName")
            .is_bot(false)
            .build();
        let real_name = Some("Real Name".to_string());

        // Act
        let mut user = User::from(member);
        user.real_name = real_name;

        // Assert
        assert_eq!(user.id, 13579);
        assert_eq!(user.nick_name, Some("".to_string())); // Should use the empty nickname
        assert_eq!(user.real_name, Some("Real Name".into()));
    }

    #[test]
    fn can_convert_server_member_when_nickname_matches_username() {
        // Arrange - when nickname matches username
        let member = ServerMemberBuilder::new()
            .id(24680)
            .nick_name("SameName")
            .user_name("SameName")
            .is_bot(false)
            .build();
        let real_name = Some("Real Name".to_string());

        // Act
        let mut user = User::from(member);
        user.real_name = real_name;

        // Assert
        assert_eq!(user.id, 24680);
        assert_eq!(user.nick_name, Some("SameName".to_string()));
        assert_eq!(user.real_name, Some("Real Name".into()));
    }

    #[test]
    fn can_format_user_with_real_name_for_display() {
        // Arrange
        let user = User {
            id: 12345,
            user_name: "UserName".to_string(),
            nick_name: Some("DisplayName".to_string()),
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
            user_name: "UserName".to_string(),
            nick_name: Some("DisplayName".to_string()),
            real_name: None,
        };

        // Act
        let display_string = format!("{}", user);

        // Assert
        assert_eq!(display_string, "UserName aka 'DisplayName'");
    }

    #[test]
    fn can_format_user_with_empty_display_name() {
        // Arrange
        let user = User {
            id: 12345,
            user_name: "UserName".to_string(),
            nick_name: Some("".to_string()),
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
            user_name: "UserName".to_string(),
            nick_name: Some("Name\"With'Quotes".to_string()),
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
            user_name: "UserName".to_string(),
            nick_name: Some("DisplayName".to_string()),
            real_name: Some("".to_string()),
        };

        // Act
        let display_string = format!("{}", user);

        // Assert
        assert_eq!(display_string, "'DisplayName' is ");
    }

    #[test]
    fn can_format_user_with_no_nickname_no_real_name() {
        // Arrange
        let user = User {
            id: 12345,
            user_name: "UserName".to_string(),
            nick_name: None,
            real_name: None,
        };

        // Act
        let display_string = format!("{}", user);

        // Assert
        assert_eq!(
            display_string,
            "UserName has neither a nickname nor a real name"
        );
    }

    #[test]
    fn can_format_user_with_real_name_but_no_nickname() {
        // Arrange
        let user = User {
            id: 12345,
            user_name: "UserName".to_string(),
            nick_name: None,
            real_name: Some("RealName".to_string()),
        };

        // Act
        let display_string = format!("{}", user);

        // Assert
        assert_eq!(display_string, "'UserName' is RealName");
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Something went wrong with Discord")]
    DiscordError(#[from] discord::Error),
    #[error("Something went wrong getting people's names")]
    NamesAccessError(#[from] names::Error),
}
