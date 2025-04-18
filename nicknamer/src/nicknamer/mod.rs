use poise::serenity_prelude as serenity;

mod config;
mod file;

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

type Reply = String;

pub struct User {
    pub(crate) name: String,
    pub(crate) id: u64,
}

pub struct RealNames {
    names: std::collections::HashMap<u64, String>,
}
pub fn reveal(
    user: &User,
    real_names: &RealNames,
) -> Result<Reply, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let real_name = real_names.names.get(&user.id).ok_or("User not found")?;
    Ok(format!(
        "Here are people's real names, {}:
         
        ",
        config::REVEAL_INSULT,
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
        real_names.names.insert(123456789, "Alice".to_string());
        real_names.names.insert(987654321, "Bob".to_string());
        real_names
    }

    #[test]
    fn test_reveal_existing_user() {
        let real_names = setup_test_data();
        let user = User {
            name: "User1".to_string(),
            id: 123456789,
        };
        let result = reveal(&user, &real_names);
        assert_eq!(result.unwrap(), "Alice".to_string());
    }

    #[test]
    fn test_reveal_different_existing_user() {
        let real_names = setup_test_data();
        let user = User {
            name: "User2".to_string(),
            id: 987654321,
        };
        let result = reveal(&user, &real_names);
        assert_eq!(result.unwrap(), "Bob".to_string());
    }

    #[test]
    fn test_reveal_nonexistent_user() {
        let real_names = setup_test_data();
        let user = User {
            name: "User3".to_string(),
            id: 111111111,
        };
        let result = reveal(&user, &real_names);
        assert!(result.is_err());
    }

    #[test]
    fn test_reveal_with_empty_names() {
        let empty_real_names = RealNames {
            names: HashMap::new(),
        };
        let user = User {
            name: "User1".to_string(),
            id: 123456789,
        };
        let result = reveal(&user, &empty_real_names);
        assert!(result.is_err());
    }
}
