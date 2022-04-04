use color_eyre::Result;
use tracing::debug;

use shbot::{config, logger, server};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    logger::install(None);
    debug!("Loaded environment variables");
    let s = std::env::var("POSTGRES_HOST")?;
    debug!("{s}");

    let cfg = config::new()?;

    server::serve(&cfg).await?;

    Ok(())
}
