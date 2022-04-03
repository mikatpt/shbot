#![allow(dead_code, unused)]

use std::collections::HashSet;

use async_trait::async_trait;
use color_eyre::eyre::eyre;
use uuid::Uuid;

use crate::{
    queue::QueueItem,
    store::{Client, Database},
    Error, Result,
};
use models::{Film, Priority, Student};

#[derive(Debug)]
pub struct MockClient {
    pub success: bool,
}

impl Database<MockClient> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for Database<MockClient> {
    fn default() -> Self {
        Self {
            client: MockClient { success: true },
        }
    }
}

#[async_trait]
impl Client for MockClient {
    // ------------- Films ------------- //

    async fn list_films(&self) -> Result<Vec<Film>> {
        if self.success {
            return Ok(vec![Film::default()]);
        }
        Err(Error::Internal(eyre!("sample error")))
    }

    async fn get_film(&self, film_name: &str) -> Result<Option<Film>> {
        if self.success {
            return Ok(Some(Film::default()));
        }
        Err(Error::Internal(eyre!("sample error")))
    }

    async fn insert_film(&self, name: &str, group: i32, priority: Priority) -> Result<Film> {
        if self.success {
            return Ok(Film::default());
        }
        Err(Error::Internal(eyre!("sample error")))
    }

    async fn update_film(&self, film: &Film) -> Result<()> {
        if self.success {
            return Ok(());
        }
        Err(Error::Internal(eyre!("sample error")))
    }

    // ------------- Junction ------------- //

    async fn get_student_films(&self, student_id: &Uuid) -> Result<HashSet<Film>> {
        Err(Error::Internal(eyre!("sample error")))
    }

    async fn insert_student_films(&self, s_id: &Uuid, f_id: &Uuid) -> Result<()> {
        Err(Error::Internal(eyre!("sample error")))
    }

    // ------------- Students ------------- //

    async fn list_students(&self) -> Result<Vec<Student>> {
        Err(Error::Internal(eyre!("sample error")))
    }

    async fn get_student(&self, slack_id: &str) -> Result<Student> {
        Err(Error::Internal(eyre!("sample error")))
    }

    #[rustfmt::skip]
    async fn insert_student_from_csv(&self, name: &str, group: i32, class: &str)
        -> Result<Student> {
        Err(Error::Internal(eyre!("sample error")))
    }
    async fn insert_student(&self, slack_id: &str) -> Result<Student> {
        Err(Error::Internal(eyre!("sample error")))
    }

    async fn update_student(&self, student: &Student) -> Result<()> {
        Err(Error::Internal(eyre!("sample error")))
    }

    // ------------- Queue ------------- //

    async fn get_queue(&self, wait: bool) -> Result<Vec<QueueItem>> {
        // if self.success {
        //     return Ok(vec![QueueItem])
        // }
        Err(Error::Internal(eyre!("sample error")))
    }

    async fn insert_to_queue(&self, q: QueueItem, wait: bool) -> Result<QueueItem> {
        Err(Error::Internal(eyre!("sample error")))
    }

    async fn delete_from_queue(&self, id: &Uuid, wait: bool) -> Result<()> {
        Err(Error::Internal(eyre!("sample error")))
    }

    fn clone(&self) -> Self {
        Self {
            success: self.success,
        }
    }

    async fn drop_db(&self) -> Result<()> {
        Err(Error::Internal(eyre!("sample error")))
    }
}
