#![allow(dead_code)]
use std::{collections::HashSet, fmt::Debug};

use async_trait::async_trait;
use deadpool_postgres::Runtime::Tokio1;
use tokio_postgres::NoTls;
use uuid::Uuid;

use crate::{
    models::{Film, Priority, Student},
    queue::QueueItem,
    Result,
};

pub mod postgres;
pub use postgres::PostgresClient;
pub mod mock;

/// Server-facing API boundary.
//
// You should implement all methods from the `Client` trait on this struct for each new client.
//
// This is not statically checked; that would require us writing and implementing an external
// trait, which just isn't done in Rust; you'd have to import the trait on every usage.
#[derive(Debug)]
pub struct Database<T: Client> {
    client: T,
}

#[async_trait]
/// Internal interface. All clients must implement this.
pub trait Client: Send + Sync + 'static {
    async fn list_films(&self) -> Result<Vec<Film>>;
    async fn get_film(&self, film_name: &str) -> Result<Option<Film>>;
    async fn insert_film(&self, name: &str, priority: Priority) -> Result<Film>;
    async fn update_film(&self, film: &Film) -> Result<()>;

    async fn get_student_films(&self, student_id: &Uuid) -> Result<HashSet<Film>>;
    async fn insert_student_films(&self, s_id: &Uuid, f_id: &Uuid) -> Result<()>;

    async fn list_students(&self) -> Result<Vec<Student>>;
    async fn get_student(&self, slack_id: &str) -> Result<Student>;
    async fn insert_student(&self, slack_id: &str) -> Result<Student>;
    async fn update_student(&self, student: &Student) -> Result<()>;

    async fn get_queue(&self, wait: bool) -> Result<Vec<QueueItem>>;
    async fn insert_to_queue(&self, q: QueueItem, wait: bool) -> Result<QueueItem>;
    async fn delete_from_queue(&self, id: &Uuid, wait: bool) -> Result<()>;

    async fn drop_db(&self) -> Result<()>;

    #[must_use]
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
    // ------------- Films ------------- //

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

    /// Updates a film.
    pub async fn update_film(&self, film: &Film) -> Result<()> {
        self.client.update_film(film).await
    }

    // ------------- Junction ------------- //

    /// Retrieves all films a student has worked on.
    pub async fn get_student_films(&self, student_id: &Uuid) -> Result<HashSet<Film>> {
        self.client.get_student_films(student_id).await
    }

    /// Inserts a shared student_film marker.
    pub async fn insert_student_films(&self, student_id: &Uuid, film_id: &Uuid) -> Result<()> {
        self.client.insert_student_films(student_id, film_id).await
    }

    // ------------- Students ------------- //

    /// Retrieve all students.
    pub async fn list_students(&self) -> Result<Vec<Student>> {
        self.client.list_students().await
    }

    /// Get a student from the database. If none, insert the student and return it.
    pub async fn get_student(&self, slack_id: &str) -> Result<Student> {
        self.client.get_student(slack_id).await
    }

    /// Insert a student. This should ONLY be called if the student isn't in the database.
    pub async fn insert_student(&self, slack_id: &str) -> Result<Student> {
        self.client.insert_student(slack_id).await
    }

    /// Updates a students information.
    pub async fn update_student(&self, student: &Student) -> Result<()> {
        self.client.update_student(student).await
    }

    // ------------- Queue ------------- //

    /// Inserts a job or student to the wait queue.
    pub async fn insert_to_queue(&self, q: QueueItem, wait: bool) -> Result<QueueItem> {
        self.client.insert_to_queue(q, wait).await
    }

    /// Gets all items from given queue.
    pub async fn get_queue(&self, wait: bool) -> Result<Vec<QueueItem>> {
        self.client.get_queue(wait).await
    }

    /// Deletes an item from the given queue.
    pub async fn delete_from_queue(&self, id: &Uuid, wait: bool) -> Result<()> {
        self.client.delete_from_queue(id, wait).await
    }

    /// Drops database. Only works in test env.
    pub async fn drop_db(&self) -> Result<()> {
        self.client.drop_db().await
    }
}

// Server-facing API implementation for Postgres.
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
