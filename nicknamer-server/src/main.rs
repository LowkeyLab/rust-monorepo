use ormlite::Connection;

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

    trait PersonRepository {
        fn get_persons(&self) -> Vec<Person>;
        fn add_person(&self, person: Person);
    }

    impl PersonController {
        fn load_persons(&self) -> Vec<Person> {
            self.repository.get_persons()
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
