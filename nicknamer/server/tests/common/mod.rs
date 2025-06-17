use migration::MigratorTrait;
use sea_orm::{Database, DatabaseConnection};
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::{postgres, testcontainers};

pub async fn setup_container() -> anyhow::Result<testcontainers::ContainerAsync<postgres::Postgres>>
{
    let container = postgres::Postgres::default().start().await?;
    Ok(container)
}

pub async fn setup_db(
    container: &testcontainers::ContainerAsync<postgres::Postgres>,
) -> anyhow::Result<DatabaseConnection> {
    let host = container.get_host().await?;
    let port = container.get_host_port_ipv4(5432).await?;
    let db_url = format!("postgres://postgres:postgres@{}:{}/postgres", host, port);
    let db = Database::connect(&db_url).await?;
    migration::Migrator::up(&db, None).await?;
    Ok(db)
}
