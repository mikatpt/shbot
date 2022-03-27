mod config;
mod logger;
mod db;
mod server;

use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    logger::install(None);

    let cfg = config::new()?;
    
    let postgres_pool = db::create_pool(&cfg.postgres);

    server::serve(&cfg).await?;

    Ok(())
}
