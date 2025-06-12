mod member {
    struct Person {
        id: i32,
        discord_id: uuid::Uuid,
        name: String,
    }

    impl Person {
        fn new(id: i32, discord_id: uuid::Uuid, name: String) -> Self {
            Person {
                id,
                discord_id,
                name,
            }
        }
    }

    struct MemberController {
        repository: Box<dyn MemberRepository>,
    }

    #[mockall::automock]
    trait MemberRepository {
        fn get_members(&self) -> Vec<Person>;
    }

    impl MemberController {
        fn load_members(&self) -> Vec<Person> {
            self.repository.get_members()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use uuid::Uuid; // Added import for Uuid

        #[test]
        fn test_member_creation() {
            let dummy_uuid = Uuid::new_v4();
            let member = Person::new(1, dummy_uuid, "Alice".to_string());
            assert_eq!(member.id, 1);
            assert_eq!(member.discord_id, dummy_uuid);
            assert_eq!(member.name, "Alice");
        }

        mod controller_tests {
            use super::*;
            // Uuid is in scope from the parent `tests` module's `use uuid::Uuid;`

            #[test]
            fn test_member_controller_load_members() {
                let mut mock_repo = MockMemberRepository::new();
                let dummy_uuid = Uuid::new_v4();
                mock_repo
                    .expect_get_members()
                    .returning(move || vec![Person::new(1, dummy_uuid, "Alice".to_string())]);

                let controller = MemberController {
                    repository: Box::new(mock_repo),
                };

                let members = controller.load_members();
                assert_eq!(members.len(), 1);
                assert_eq!(members[0].id, 1);
                assert_eq!(members[0].discord_id, dummy_uuid);
                assert_eq!(members[0].name, "Alice");
            }
        }
    }
}
fn main() {
    println!("Nicknamer server is running...");
}
