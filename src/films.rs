use std::str::FromStr;

use crate::{models::Priority, store::Client, store::Database, Error, Result};

use futures::{future, stream::FuturesUnordered};
use tracing::{error, info};

pub(crate) struct FilmManager<T: Client> {
    db: Database<T>,
}

impl<T: Client> FilmManager<T> {
    pub(crate) fn new(db: Database<T>) -> Self {
        Self { db }
    }

    /// Message format: "PRIORITY film1 film2 film3"
    /// Returns slack text detailing # of inserts.
    pub async fn insert_films(&self, message: &str) -> Result<String> {
        use std::collections::HashSet;
        let (priority, film_names) = message.trim().split_once(' ').unwrap_or_default();

        info!("Inserting {priority} priority films: {film_names:?}");

        let priority = Priority::from_str(priority)?;
        let film_names: Vec<_> = film_names
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        // Concurrently insert all films
        let films: FuturesUnordered<_> = film_names
            .clone()
            .into_iter()
            .map(|film_name| {
                let db = self.db.clone();
                tokio::spawn(async move {
                    db.insert_film(&film_name, priority)
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

        let v = v?;
        let successes: HashSet<_> = v
            .iter()
            .filter(|r| r.is_ok())
            .map(|r| {
                if let Ok(res) = r {
                    return res.name.as_str();
                }
                unreachable!()
            })
            .collect();

        let fails = film_names
            .iter()
            .filter(|n| !successes.contains(n.as_str()));

        let mut msg = String::new();

        if !successes.is_empty() {
            msg += &format!("Successfully inserted {} films(s):\n", successes.len());
            successes.iter().for_each(|&s| {
                msg += s;
                msg += ", ";
            });
            msg.pop();
            msg.pop();
        }

        if film_names.len() - successes.len() > 0 {
            if !successes.is_empty() {
                msg += "\n";
            }
            msg += "Some films were duplicates, so skipped:\n";
            fails.for_each(|s| {
                msg += s;
                msg += ", ";
            });
            msg.pop();
            msg.pop();
        }

        Ok(msg)
    }
}
