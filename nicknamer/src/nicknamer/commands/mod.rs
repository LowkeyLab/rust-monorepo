use poise::serenity_prelude as serenity;
use std::fmt::{Display, Formatter};
pub mod reveal;

type Reply = String;

#[derive(Debug, PartialEq)]
pub struct User {
    pub id: u64,
    pub display_name: String,
    pub real_name: String,
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}' is {}", self.display_name, self.real_name)
    }
}

#[derive(Debug, PartialEq)]
pub struct RealNames {
    pub(crate) users: Vec<User>,
}

impl Display for RealNames {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.users
                .iter()
                .map(|user| format!("{}", user))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

#[allow(dead_code)]
pub fn nick(_user_id: serenity::UserId) {}
