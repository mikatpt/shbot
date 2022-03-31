use std::fmt::Debug;

use async_trait::async_trait;
use deadpool_postgres::{Pool, Runtime::Tokio1};
use tokio_postgres::NoTls;

pub(crate) mod films;
use crate::{
    models::{Film, Priority},
    Result,
};

/// Returns a new Postgres database instance.
pub(crate) fn new(cfg: &deadpool_postgres::Config) -> Result<Database<PostgresClient>> {
    let pool = cfg.create_pool(Some(Tokio1), NoTls)?;
    Ok(Database::new(pool))
}

/// Server-facing API boundary.
//
// You should implement all methods from the `Client` trait on this struct for each new client.
//
// This is not statically checked; that would require us writing and implementing an external
// trait, which just isn't done in Rust; you'd have to import the trait on every usage.
#[derive(Debug)]
pub(crate) struct Database<C: Client> {
    client: C,
}

#[async_trait]
/// Internal interface. All clients must implement this.
pub(crate) trait Client {
    /// Retrieves all films.
    async fn list_films(&self) -> Result<Vec<Film>>;

    /// Retrieves a film given its name.
    async fn get_film(&self, film_name: &str) -> Result<Option<Film>>;

    /// Inserts an empty film with no roles worked.
    async fn insert_film(&self, name: &str, priority: Priority) -> Result<Film>;

    /// Updates a film
    async fn update_film(&self, film: &Film) -> Result<()>;
}

/// Internal Postgres client.
pub(crate) struct PostgresClient {
    pool: Pool,
}

// Server-facing API implementation for Postgres.
// Implementation is split between files on model boundaries.
impl Database<PostgresClient> {
    pub fn new(pool: Pool) -> Self {
        let client = PostgresClient { pool };
        Database { client }
    }
}

// Axum requires that we implement debug to use this in state.
impl Debug for PostgresClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostgresClient")
            .field("pool", &"<pool>")
            .finish()
    }
}
