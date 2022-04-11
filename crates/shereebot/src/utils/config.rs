use std::{env, net::SocketAddr};

use color_eyre::Result;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub server: Server,
    pub postgres: deadpool_postgres::Config,
    pub token: String,
}

#[derive(Deserialize)]
pub struct Server {
    pub address: SocketAddr,
    pub port: String,
}

pub fn new() -> Result<Config> {
    let port = env::var("SERVER_PORT")?;
    let address = SocketAddr::from(([0, 0, 0, 0], port.parse()?));
    let pg_port = env::var("POSTGRES_PORT")?.parse::<u16>()?;

    let server = Server { port, address };
    let postgres = deadpool_postgres::Config {
        user: env::var("POSTGRES_USER").ok(),
        host: env::var("POSTGRES_HOST").ok(),
        password: env::var("POSTGRES_PASSWORD").ok(),
        dbname: env::var("POSTGRES_DBNAME").ok(),
        port: Some(pg_port),
        ..Default::default()
    };
    let token = env::var("OAUTH_TOKEN")?;

    Ok(Config {
        server,
        postgres,
        token,
    })
}
