use std::env;
use std::net::SocketAddr;

use color_eyre::Result;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub server: Server,
    pub postgres: deadpool_postgres::Config,
}

#[derive(Deserialize)]
pub struct Server {
    pub address: SocketAddr,
}

pub fn new() -> Result<Config> {
    let port = env::var("SERVER_PORT")?;
    let address = SocketAddr::from(([0, 0, 0, 0], port.parse()?));

    let server = Server { address };
    let postgres = deadpool_postgres::Config {
        user: env::var("POSTGRES_USER").ok(),
        host: env::var("POSTGRES_HOST").ok(),
        password: env::var("POSTGRES_PASSWORD").ok(),
        dbname: env::var("POSTGRES_DBNAME").ok(),
        ..Default::default()
    };

    Ok(Config { server, postgres })
}
