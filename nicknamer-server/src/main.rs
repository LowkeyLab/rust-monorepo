mod person {
    #[derive(ormlite::Model, Debug)]
    #[ormlite(insert = "InsertPerson")]
    struct Person {
        #[ormlite(primary_key)]
        id: i32,
        discord_id: i32,
        name: String,
    }

    impl Person {
        fn new(discord_id: i32, name: String) -> Self {
            Person {
                id: 0, // Set default id to 0
                discord_id,
                name,
            }
        }
    }

    struct PersonController {
        repository: Box<dyn PersonRepository>,
    }

    #[mockall::automock]
    trait PersonRepository {
        fn get_members(&self) -> Vec<Person>;
    }

    impl PersonController {
        fn load_members(&self) -> Vec<Person> {
            self.repository.get_members()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_member_creation() {
            let dummy_discord_id = 123456789;
            let member = Person::new(dummy_discord_id, "Alice".to_string());
            assert_eq!(member.id, 0); // Expect default id of 0
            assert_eq!(member.discord_id, dummy_discord_id);
            assert_eq!(member.name, "Alice");
        }

        mod controller_tests {
            use super::*;
            // Uuid is in scope from the parent `tests` module's `use uuid::Uuid;`

            #[test]
            fn test_member_controller_load_members() {
                let mut mock_repo = MockPersonRepository::new();
                let dummy_discord_id = 123456789;
                mock_repo
                    .expect_get_members()
                    .returning(move || vec![Person::new(dummy_discord_id, "Alice".to_string())]);

                let controller = PersonController {
                    repository: Box::new(mock_repo),
                };

                let members = controller.load_members();
                assert_eq!(members.len(), 1);
                assert_eq!(members[0].id, 0); // Expect default id of 0
                assert_eq!(members[0].discord_id, dummy_discord_id);
                assert_eq!(members[0].name, "Alice");
            }
        }
    }
}
fn main() {
    println!("Nicknamer server is running...");
}
