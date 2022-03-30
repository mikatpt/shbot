use color_eyre::Result;
use tracing::debug;

use shbot::{config, logger, server};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    logger::install(None);
    debug!("Loaded environment variables");

    let cfg = config::new()?;

    server::serve(&cfg).await?;

    Ok(())
}
