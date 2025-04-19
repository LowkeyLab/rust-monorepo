use crate::nicknamer::commands::{RealNames, Reply, User};
use crate::nicknamer::config;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub fn reveal(real_names: &RealNames) -> Result<Reply, Error> {
    let users = real_names
        .users
        .iter()
        .filter(|user| user.real_name.is_some())
        .map(|user| user.to_string())
        .collect::<Vec<String>>();
    Ok(format!(
        "Here are people's real names, {}:
{}",
        config::REVEAL_INSULT,
        users.join("\n")
    ))
}

pub fn reveal_user(user: User) -> Result<Reply, Error> {
    match user.real_name {
        Some(_) => Ok(user.to_string()),
        None => Ok(format!(
            "How mysterious! {}'s true name is shrouded by darkness",
            user.display_name
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::nicknamer::commands::reveal;
    use crate::nicknamer::commands::*;
    use crate::nicknamer::config;

    fn setup_test_data() -> RealNames {
        let mut users = Vec::new();

        users.push(User {
            id: 1,
            display_name: "Alice's nickname".to_string(),
            real_name: Some("Alice".to_string()),
        });

        users.push(User {
            id: 2,
            display_name: "Bob's nickname".to_string(),
            real_name: Some("Bob".to_string()),
        });

        RealNames { users }
    }

    #[test]
    fn can_reveal_users_with_real_names() {
        let real_names = setup_test_data();
        let result = reveal::reveal(&real_names).unwrap();

        // Check that the output starts with the expected header
        let header = format!("Here are people's real names, {}:", config::REVEAL_INSULT);
        assert!(
            result.starts_with(&header),
            "Result should start with the header"
        );

        // Check for each expected user string in the output
        assert!(
            result.contains("'Alice's nickname' is Alice"),
            "Result should contain Alice's information"
        );
        assert!(
            result.contains("'Bob's nickname' is Bob"),
            "Result should contain Bob's information"
        );
    }

    #[test]
    fn can_reveal_users_even_if_there_are_no_real_names() {
        let empty_real_names = RealNames { users: Vec::new() };
        let result = reveal::reveal(&empty_real_names).unwrap();

        // For empty names, we can still do an exact match as there's no ordering issue
        assert_eq!(
            result,
            format!("Here are people's real names, {}:\n", config::REVEAL_INSULT)
        );
    }

    #[test]
    fn can_reveal_single_user() {
        // Test for an existing member
        let existing_result = reveal::reveal_user(User {
            id: 0,
            display_name: "Alice's nickname".to_string(),
            real_name: Some("Alice".to_string()),
        })
        .unwrap();
        assert_eq!(existing_result, "'Alice's nickname' is Alice");
    }

    #[test]
    fn revealing_user_with_no_nickname_results_in_insult() {
        // Test for a user with no real name
        let result = reveal::reveal_user(User {
            id: 0,
            display_name: "Unknown User".to_string(),
            real_name: None,
        })
        .unwrap();

        // The Display implementation for User should return an empty string when real_name is None
        assert_eq!(
            result,
            "How mysterious! Unknown User's true name is shrouded by darkness"
        );
    }
}
