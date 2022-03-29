use deadpool_postgres::{Client, Config, Pool, PoolError, Runtime::Tokio1};
use tokio_postgres::NoTls;

pub fn create_pool(cfg: &Config) -> Pool {
    cfg.create_pool(Some(Tokio1), NoTls).unwrap()
}

/// Inserts a film into the database.
pub async fn insert_film(pool: &Pool, film_name: &str) -> color_eyre::Result<(), PoolError> {
    let client: Client = pool.get().await?;

    let query = "
        DO $$
        DECLARE roles_id roles.id%TYPE;
        BEGIN
            INSERT INTO roles DEFAULT VALUES RETURNING id INTO roles_id;
            INSERT INTO films(name, roles_id) VALUES ($1, roles_id);
        END $$;";

    let statement = client.prepare_cached(query).await?;

    client.execute(&statement, &[&film_name]).await?;

    Ok(())
}
