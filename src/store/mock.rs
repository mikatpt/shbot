#![allow(dead_code, unused)]

use async_trait::async_trait;
use color_eyre::eyre::eyre;

use crate::{
    models::{Film, Priority},
    store::{Client, Database},
    Error, Result,
};

#[derive(Debug)]
pub struct MockClient {
    pub success: bool,
}

impl Database<MockClient> {
    pub fn new() -> Self {
        Self {
            client: MockClient { success: true },
        }
    }
}

#[async_trait]
impl Client for MockClient {
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

    async fn insert_film(&self, name: &str, priority: Priority) -> Result<Film> {
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

    fn clone(&self) -> Self {
        Self {
            success: self.success,
        }
    }
}
