mod config;
mod logger;
mod db;
mod server;
mod interceptors;

use color_eyre::Result;
use tracing::debug;

#[tokio::main]
async fn main() -> Result<()> {
    debug!("Loading environment variables...");
    dotenv::dotenv().ok();
    debug!("Environment loaded!");
    logger::install(None);

    let cfg = config::new()?;

    server::serve(&cfg).await?;

    Ok(())
}
