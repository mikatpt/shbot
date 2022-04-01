use std::str::FromStr;

use futures::{future, stream::FuturesUnordered};
use itertools::Itertools;
use tracing::{error, info};

use crate::{models::Priority, store::Client, store::Database, Error, Result};

pub(crate) struct FilmManager<T: Client> {
    db: Database<T>,
}

const PRI_ERR: &str = "I couldn't read your command :cry:
Valid priority weights are `HIGH` and `LOW`.
Ex: `insert-films HIGH film1, film2, film3`";

impl<T: Client> FilmManager<T> {
    pub(crate) fn new(db: Database<T>) -> Self {
        Self { db }
    }

    /// Message format: "PRIORITY film1 film2 film3"
    /// Returns slack text detailing # of inserts.
    pub async fn insert_films(&self, message: &str) -> String {
        use std::collections::HashSet;
        dbg!(message);
        let (priority, film_names) = message.trim().split_once(' ').unwrap_or_default();

        info!("Inserting {priority} priority films: {film_names:?}");

        dbg!(priority);
        let priority = match Priority::from_str(priority) {
            Ok(p) => p,
            Err(_) => return PRI_ERR.to_string(),
        };

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

        let v = match v {
            Ok(r) => r,
            Err(e) => return e.to_string(),
        };

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
            .filter(|n| !successes.contains(n.as_str()))
            .map(|s| s.as_ref());

        let mut msg = String::new();

        if !successes.is_empty() {
            msg += &format!("Successfully inserted {} films(s)!\n", successes.len());
            successes.iter().intersperse(&", ").for_each(|s| msg += s);
        }

        if film_names.len() - successes.len() > 0 {
            if !successes.is_empty() {
                msg += "\n";
            }
            msg += "Skipped duplicate films:\n";
            fails.intersperse(", ").for_each(|s| msg += s);
        }

        msg
    }
}
