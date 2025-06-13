use ormlite::Connection;

mod person {
    #[derive(ormlite::Model, Debug)]
    #[ormlite(insert = "InsertUser")]
    struct User {
        #[ormlite(primary_key)]
        id: i32,
        discord_id: i32,
        name: String,
    }

    impl User {
        fn new(discord_id: i32, name: String) -> Self {
            User {
                id: 0, // Set default id to 0
                discord_id,
                name,
            }
        }
    }

    struct UserController {
        repository: Box<dyn UserRepository>,
    }

    trait UserRepository {
        fn get_users(&self) -> Vec<User>;
        fn add_user(&self, user: User);
    }

    impl UserController {
        fn load_users(&self) -> Vec<User> {
            self.repository.get_users()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_user_creation() {
            let dummy_discord_id = 123456789;
            let user = User::new(dummy_discord_id, "Alice".to_string());
            assert_eq!(user.id, 0); // Expect default id of 0
            assert_eq!(user.discord_id, dummy_discord_id);
            assert_eq!(user.name, "Alice");
        }
    }
}

#[tokio::main]
async fn main() {
    let connection = ormlite::postgres::PgConnection::connect(
        "postgres://username:password@localhost/nicknamer",
    )
    .await
    .expect("Failed to connect to the database");
    println!("Nicknamer server is running...");
}
