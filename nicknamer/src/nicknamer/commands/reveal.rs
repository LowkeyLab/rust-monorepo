use crate::nicknamer::commands::{RealNames, Reply};
use crate::nicknamer::config;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub fn reveal(real_names: &RealNames) -> Result<Reply, Error> {
    Ok(format!(
        "Here are people's real names, {}:
{}",
        config::REVEAL_INSULT,
        real_names
    ))
}

pub fn reveal_member(id: u64, real_names: &RealNames) -> Result<Reply, Error> {
    match real_names.users.get(&id) {
        Some(user) => Ok(format!("'{}' is {}", user.display_name, user.real_name)),
        None => Ok("No one has that id".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use crate::nicknamer::commands::reveal;
    use crate::nicknamer::commands::*;
    use crate::nicknamer::config;
    use std::collections::HashMap;

    fn setup_test_data() -> RealNames {
        let mut users = HashMap::new();

        users.insert(
            1,
            User {
                id: 1,
                display_name: "Alice's nickname".to_string(),
                real_name: "Alice".to_string(),
            },
        );

        users.insert(
            2,
            User {
                id: 2,
                display_name: "Bob's nickname".to_string(),
                real_name: "Bob".to_string(),
            },
        );

        RealNames { users }
    }

    #[test]
    fn test_reveal_existing_user() {
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
    fn test_reveal_empty_names() {
        let empty_real_names = RealNames {
            users: HashMap::new(),
        };
        let result = reveal::reveal(&empty_real_names).unwrap();

        // For empty names, we can still do an exact match as there's no ordering issue
        assert_eq!(
            result,
            format!("Here are people's real names, {}:\n", config::REVEAL_INSULT)
        );
    }

    #[test]
    fn test_reveal_member() {
        let real_names = setup_test_data();

        // Test for an existing member
        let existing_result = reveal::reveal_member(1, &real_names).unwrap();
        assert_eq!(existing_result, "'Alice's nickname' is Alice");

        // Test for a non-existent member
        let nonexistent_result = reveal::reveal_member(999, &real_names).unwrap();
        assert_eq!(nonexistent_result, "No one has that id");
    }
}
