use crate::nicknamer::commands::{RealNames, Reply, User};
use crate::nicknamer::discord::{ServerMember, serenity};
use crate::nicknamer::{config, file};

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

pub fn reveal_member(
    server_member: ServerMember,
    real_names: &file::RealNames,
) -> Result<Reply, serenity::Error> {
    let user_id = server_member.id;
    let mut user: User = server_member.into();
    let real_name = real_names.names.get(&user_id).cloned();
    user.real_name = real_name;
    create_reply(user)
}

fn create_reply(user: User) -> Result<Reply, Error> {
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
    use crate::nicknamer::discord::ServerMember;
    use crate::nicknamer::file::RealNames;
    use std::collections::HashMap;

    fn create_test_real_names() -> RealNames {
        let mut names = HashMap::new();
        names.insert(123456789, "Alice".to_string());
        names.insert(987654321, "Bob".to_string());
        RealNames { names }
    }

    fn create_server_member(id: u64, nickname: Option<String>, username: String) -> ServerMember {
        ServerMember {
            id,
            nick_name: nickname,
            user_name: username,
        }
    }

    #[test]
    fn test_reveal_member_with_real_name() {
        // Setup
        let real_names = create_test_real_names();
        let server_member = create_server_member(
            123456789,
            Some("AliceNickname".to_string()),
            "AliceUsername".to_string(),
        );

        // Call the function
        let result = reveal::reveal_member(server_member, &real_names).unwrap();

        // The result should contain the user's nickname (or username if no nickname) and real name
        assert_eq!(result, "'AliceNickname' is Alice");
    }

    #[test]
    fn test_reveal_member_without_nickname() {
        // Setup - member with username but no nickname
        let real_names = create_test_real_names();
        let server_member = create_server_member(123456789, None, "AliceUsername".to_string());

        // Call the function
        let result = reveal::reveal_member(server_member, &real_names).unwrap();

        // Should use the username when no nickname is available
        assert_eq!(result, "'AliceUsername' is Alice");
    }

    #[test]
    fn test_reveal_member_without_real_name() {
        // Setup - member with an ID that doesn't exist in real_names
        let real_names = create_test_real_names();
        let server_member = create_server_member(
            111222333,
            Some("UnknownNickname".to_string()),
            "UnknownUsername".to_string(),
        );

        // Call the function
        let result = reveal::reveal_member(server_member, &real_names).unwrap();

        // Should return the "mysterious" message for users without real names
        assert_eq!(
            result,
            "How mysterious! UnknownNickname's true name is shrouded by darkness"
        );
    }

    #[test]
    fn test_reveal_member_with_special_characters() {
        // Setup
        let mut real_names = create_test_real_names();
        real_names
            .names
            .insert(555666777, "User with \"quotes\"".to_string());
        let server_member = create_server_member(
            555666777,
            Some("Nickname'With'Apostrophes".to_string()),
            "Username".to_string(),
        );

        // Call the function
        let result = reveal::reveal_member(server_member, &real_names).unwrap();

        // Should handle special characters correctly
        assert_eq!(
            result,
            "'Nickname'With'Apostrophes' is User with \"quotes\""
        );
    }

    #[test]
    fn test_reveal_member_preserves_nickname_case() {
        // Setup
        let real_names = create_test_real_names();
        let server_member = create_server_member(
            123456789,
            Some("ALICE_UPPERCASE".to_string()),
            "alice_lowercase".to_string(),
        );

        // Call the function
        let result = reveal::reveal_member(server_member, &real_names).unwrap();

        // Should preserve the nickname case exactly
        assert_eq!(result, "'ALICE_UPPERCASE' is Alice");
    }

    #[test]
    fn test_multiple_members_with_same_real_name() {
        // Setup - two members with the same real name
        let mut real_names = create_test_real_names();
        real_names.names.insert(111222333, "Bob".to_string());

        // First member
        let server_member1 = create_server_member(
            987654321,
            Some("BobNickname1".to_string()),
            "BobUsername1".to_string(),
        );
        let result1 = reveal::reveal_member(server_member1, &real_names).unwrap();

        // Second member
        let server_member2 = create_server_member(
            111222333,
            Some("BobNickname2".to_string()),
            "BobUsername2".to_string(),
        );
        let result2 = reveal::reveal_member(server_member2, &real_names).unwrap();

        // Different nicknames but same real name
        assert_eq!(result1, "'BobNickname1' is Bob");
        assert_eq!(result2, "'BobNickname2' is Bob");
    }

    #[test]
    fn test_conversion_from_server_member_to_user() {
        // Setup
        let server_member = create_server_member(
            123456789,
            Some("Nickname".to_string()),
            "Username".to_string(),
        );

        // Convert to User manually as reveal_member would do
        let user: User = server_member.into();

        // Verify conversion worked as expected
        assert_eq!(user.id, 123456789);
        assert_eq!(user.display_name, "Nickname");
        assert_eq!(user.real_name, None); // real_name should be None initially
    }

    #[test]
    fn revealing_user_with_no_nickname_results_in_insult() {
        // Test for a user with no real name
        let result = reveal::create_reply(User {
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
