use std::str::FromStr;

use crate::{
    models::{Film, Priority},
    server::State,
    Error, Result,
};

use futures::{future, stream::FuturesUnordered};
use tracing::{error, info};

pub struct FilmManager {
    state: State,
}

impl FilmManager {
    pub(crate) fn new(state: State) -> Self {
        Self { state }
    }

    /// Message format: "PRIORITY film1 film2 film3"
    pub async fn insert_films(&self, message: &str) -> Result<Vec<Result<Film>>> {
        let (priority, film_names) = message.trim().split_once(' ').unwrap_or_default();

        info!("Inserting {priority} priority films: {film_names:?}");

        let priority = Priority::from_str(priority)?;

        // Concurrently insert all films
        let films: FuturesUnordered<_> = film_names
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .map(|film_name| {
                let s = self.state.clone();
                tokio::spawn(async move {
                    s.db.insert_film(&film_name, priority)
                        .await
                        .map_err(|e| -> Error {
                            error!("{e}");
                            e
                        })
                })
            })
            .collect();

        // Await all inserts to complete.
        let v: Result<Vec<_>> = future::join_all(films)
            .await
            .into_iter()
            .map(|r| r.map_err(Into::<Error>::into))
            .collect();
        v
    }
}
