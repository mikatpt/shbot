use std::fmt::Debug;

use async_trait::async_trait;
use deadpool_postgres::Runtime::Tokio1;
use tokio_postgres::NoTls;

use crate::{
    models::{Film, Priority},
    Result,
};

pub(crate) mod postgres;
pub(crate) use postgres::PostgresClient;
pub mod mock;

/// Server-facing API boundary.
//
// You should implement all methods from the `Client` trait on this struct for each new client.
//
// This is not statically checked; that would require us writing and implementing an external
// trait, which just isn't done in Rust; you'd have to import the trait on every usage.
#[derive(Debug)]
pub(crate) struct Database<T: Client> {
    client: T,
}

#[async_trait]
/// Internal interface. All clients must implement this.
pub(crate) trait Client: Send + Sync + 'static {
    async fn list_films(&self) -> Result<Vec<Film>>;

    async fn get_film(&self, film_name: &str) -> Result<Option<Film>>;

    async fn insert_film(&self, name: &str, priority: Priority) -> Result<Film>;

    async fn update_film(&self, film: &Film) -> Result<()>;

    fn clone(&self) -> Self;
}

impl<T: Client> Clone for Database<T> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
        }
    }
}

impl<T: Client> Database<T> {
    /// Retrieves all films.
    pub async fn list_films(&self) -> Result<Vec<Film>> {
        self.client.list_films().await
    }

    /// Retrieves a film given its name.
    pub async fn get_film(&self, film_name: &str) -> Result<Option<Film>> {
        self.client.get_film(film_name).await
    }

    /// Inserts an empty film with no roles worked.
    pub async fn insert_film(&self, name: &str, priority: Priority) -> Result<Film> {
        self.client.insert_film(name, priority).await
    }

    /// Updates a film
    pub async fn update_film(&self, film: &Film) -> Result<()> {
        self.client.update_film(film).await
    }
}

// Server-facing API implementation for Postgres.
// Implementation is split between files on model boundaries.
impl Database<PostgresClient> {
    /// Returns a new Postgres database instance.
    pub fn new(cfg: &deadpool_postgres::Config) -> Result<Self> {
        let pool = cfg.create_pool(Some(Tokio1), NoTls)?;
        let client = PostgresClient::new(pool);
        Ok(Database { client })
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
