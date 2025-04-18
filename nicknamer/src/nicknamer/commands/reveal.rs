use crate::nicknamer::commands::{RealNames, Reply};
use crate::nicknamer::config;

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
    use crate::nicknamer::commands::reveal;
    use crate::nicknamer::commands::*;
    use crate::nicknamer::config;

    fn setup_test_data() -> RealNames {
        RealNames {
            users: vec![
                User {
                    id: 1,
                    display_name: "Alice's nickname".to_string(),
                    real_name: "Alice".to_string(),
                },
                User {
                    id: 2,
                    display_name: "Bob's nickname".to_string(),
                    real_name: "Bob".to_string(),
                },
            ],
        }
    }

    #[test]
    fn test_reveal_existing_user() {
        let real_names = setup_test_data();
        let result = reveal::reveal(&real_names);
        assert_eq!(
            result.unwrap(),
            format!(
                "Here are people's real names, {}:\n            Alice's nickname: Alice\nBob's nickname: Bob\n        ",
                config::REVEAL_INSULT
            )
        );
    }

    #[test]
    fn test_reveal_empty_names() {
        let empty_real_names = RealNames { users: Vec::new() };
        let result = reveal::reveal(&empty_real_names);
        assert_eq!(
            result.unwrap(),
            format!(
                "Here are people's real names, {}:\n            \n        ",
                config::REVEAL_INSULT
            )
        );
    }
}
