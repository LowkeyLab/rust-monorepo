pub use sea_orm_migration::prelude::*;

mod m20250822_000001_create_players_table;
mod m20250822_000002_update_games_table;
mod m20250822_000003_create_game_players_table;
mod m20250822_000004_create_rounds_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250822_000001_create_players_table::Migration),
            Box::new(m20250822_000002_update_games_table::Migration),
            Box::new(m20250822_000003_create_game_players_table::Migration),
            Box::new(m20250822_000004_create_rounds_table::Migration),
        ]
    }
}
