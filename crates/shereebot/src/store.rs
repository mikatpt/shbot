#![allow(dead_code)]
use std::collections::HashSet;

use async_trait::async_trait;
use deadpool_postgres::Runtime::Tokio1;
use tokio_postgres::NoTls;
use uuid::Uuid;

use crate::{queue::QueueItem, Result};
use models::{Film, Priority, Role, Student};

pub mod postgres;
pub use postgres::PostgresClient;

pub mod mock;

/// Server-facing API boundary.
pub type Database = Box<dyn Client>;

#[async_trait]
/// Internal database interface.
///
/// This requires that your database client use some sort of synchronization primitive to allow for
/// sending across threads (ergo the `Send + Sync + 'static` bounds.)
pub trait Client: CloneClient + Send + Sync + 'static {
    /// Retrieves all films.
    async fn list_films(&self) -> Result<Vec<Film>>;
    /// Retrieves a film given its name.
    async fn get_film(&self, film_name: &str) -> Result<Option<Film>>;
    /// Inserts an empty film with no roles worked.
    async fn insert_film(&self, name: &str, group_number: i32, priority: Priority) -> Result<Film>;
    /// Updates a film.
    async fn update_film(&self, film: &Film) -> Result<()>;

    /// Retrieves all films a student has worked on.
    async fn get_worked_films(&self, student_id: &Uuid) -> Result<HashSet<Film>>;
    /// Inserts a shared student_film marker.
    async fn insert_student_films(&self, s_id: &Uuid, f_id: &Uuid) -> Result<()>;
    /// Gets all films where given role is available AND student is not in its group.
    async fn get_films_exclusionary(&self, group: i32, role: Role) -> Result<Vec<Film>>;

    /// Retrieve all students.
    async fn list_students(&self) -> Result<Vec<Student>>;
    /// Get a student from the database. If none, insert the student and return it.
    async fn get_student(&self, slack_id: &str) -> Result<Student>;
    /// From csv upload
    async fn insert_student_from_csv(&self, name: &str, group: i32, class: &str)
        -> Result<Student>;
    /// Insert a student. This should ONLY be called if the student isn't in the database.
    async fn insert_student(&self, slack_id: &str, name: &str) -> Result<Student>;
    /// Updates a students information.
    async fn update_student(&self, student: &Student) -> Result<()>;

    /// Inserts a job or student to the wait queue.
    async fn get_queue(&self, wait: bool) -> Result<Vec<QueueItem>>;
    /// Gets all items from given queue.
    async fn insert_to_queue(&self, q: QueueItem, wait: bool) -> Result<QueueItem>;
    /// Deletes an item from the given queue.
    async fn delete_from_queue(&self, id: &Uuid, wait: bool) -> Result<()>;

    /// Drops database. Only works in test env.
    async fn drop_db(&self) -> Result<()>;
}

/// Workaround to allow cloning trait.
pub trait CloneClient {
    fn clone_box(&self) -> Database;
}

impl<T: 'static + Client + Clone> CloneClient for T {
    fn clone_box(&self) -> Database {
        Box::new(self.clone())
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Client>")
    }
}

// Server-facing API implementation for Postgres.
pub fn new(cfg: &deadpool_postgres::Config) -> Result<Database> {
    let pool = cfg.create_pool(Some(Tokio1), NoTls)?;
    let client = Box::new(PostgresClient::new(pool));
    Ok(client)
}

pub fn new_mock() -> Database {
    Box::new(mock::MockClient { success: true })
}
