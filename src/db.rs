use deadpool_postgres::{Config, Client, Pool, PoolError, Runtime::Tokio1};

pub fn create_pool(cfg: &Config) -> Pool {
    cfg.create_pool(Some(Tokio1), tokio_postgres::NoTls).unwrap()
}

pub async fn insert_stat(pool: &Pool, command: &str) -> color_eyre::Result<(), PoolError> {
    let client: Client = pool.get().await?;

    let statement = client.prepare_cached(
        "INSERT INTO cli_stats(command, count) 
         VALUES($1, 1) 
         ON CONFLICT (command) DO
         UPDATE SET count = cli_stats.count + 1;"
    ).await?;

    client.execute(&statement, &[&command]).await?;

    Ok(())
}
