use crate::nicknamer::config;
use poise::serenity_prelude as serenity;
use std::fmt::{Display, Formatter};

type Reply = String;

pub struct User {
    display_name: String,
    real_name: String,
}
#[derive(Debug, PartialEq)]
pub struct RealNames {
    pub(crate) names: std::collections::HashMap<String, String>,
}

impl Display for RealNames {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.names
                .iter()
                .map(|(id, name)| format!("{}: {}", id, name))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

///    This function handles the 'nick' command for the `nicknamer` bot. Its purpose is to
///     allow discord users to manage each other's nicknames, even if they are in the same
///     Discord Role.
///
///     The bot applies any nickname changes as specified by this command.
///
///     This command assumes that the bot has a higher Role than all users which invoke this
///     command.
///
///     In certain failure scenarios, such as offering an invalid nickname, the bot will
///     reply with information about the invalid command.
#[allow(dead_code)]
pub fn nick(_user_id: serenity::UserId) {}

pub fn reveal(
    real_names: &RealNames,
) -> Result<Reply, Box<dyn std::error::Error + Send + Sync + 'static>> {
    Ok(format!(
        "Here are people's real names, {}:
            {}
        ",
        config::REVEAL_INSULT,
        real_names
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn setup_test_data() -> RealNames {
        let mut real_names = RealNames {
            names: HashMap::new(),
        };
        real_names
            .names
            .insert("Alice's nickname".to_string(), "Alice".to_string());
        real_names
            .names
            .insert("Bob's nickname".to_string(), "Bob".to_string());
        real_names
    }
    #[test]
    fn test_reveal_existing_user() {
        let real_names = setup_test_data();
        let result = reveal(&real_names);
        assert_eq!(
            result.unwrap(),
            format!(
                "Here are people's real names, {}:\n            Bob's nickname: Bob\nAlice's nickname: Alice\n        ",
                config::REVEAL_INSULT
            )
        );
    }

    #[test]
    fn test_reveal_empty_names() {
        let empty_real_names = RealNames {
            names: HashMap::new(),
        };
        let result = reveal(&empty_real_names);
        assert_eq!(
            result.unwrap(),
            format!(
                "Here are people's real names, {}:\n            \n        ",
                config::REVEAL_INSULT
            )
        );
    }
}
