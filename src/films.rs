#![allow(dead_code)]
use std::{collections::HashSet, str::FromStr};

use futures::{future, stream::FuturesUnordered};
use itertools::Itertools;
use tracing::{error, info};

use crate::{
    models::Priority, server::State, slack::app_mentions::Response, store::Client, Error, Result,
};

pub(crate) struct FilmManager<T: Client> {
    state: State<T>,
}

const DELIVER: &str = "Good job!! You've delivered your work.
When you're ready to pick up another job, just type @ShereeBot request-work.
Then, I'll message you back when there's a job ready for you.";

const INTERNAL_ERR: &str = "Something went wrong internally - please let Sheree know!";

const PRI_ERR: &str = "I couldn't read your command :cry:
Valid priority weights are `HIGH` and `LOW`.
Ex: `insert-films HIGH film1, film2, film3`";

const NO_WORK: &str = "No work is available yet :cry:
I'll reply in this thread once I find some work for you!";

impl<T: Client> FilmManager<T> {
    pub(crate) fn new(state: State<T>) -> Self {
        Self { state }
    }

    /// When a request comes in, polls the jobs queue for work to assign.
    /// Returns a formatted response to send back to the user
    pub async fn request_work(&self, slack_id: &str, ts: &str, channel: &str) -> String {
        match self.state.queue.try_assign_job(slack_id, ts, channel).await {
            Ok(Some(j)) => format!(
                "You've been assigned to work {} on {}!",
                j.role.as_ref(),
                j.film_name
            ),
            Ok(None) => NO_WORK.to_string(),
            Err(err) => {
                if let Error::Duplicate(_) = err {
                    "You're all done! No more work for you :)".to_string()
                } else {
                    error!("Error when requesting work: {}", err);
                    INTERNAL_ERR.to_string()
                }
            }
        }
    }

    /// Deliver work. On success, attempts to assign out jobs to the wait queue.
    pub async fn deliver_work(&self, slack_id: &str) -> String {
        let student = match self.state.db.get_student(slack_id).await {
            Ok(u) => u,
            Err(e) => return report_error(e),
        };
        match self.state.queue.deliver(student, slack_id).await {
            Ok(_) => {
                // After delivering the work, we'll try to assign jobs out to the wait queue.
                let s = self.state.clone();
                tokio::spawn(async move {
                    match s.queue.try_empty_wait_queue().await {
                        Ok(jobs) => {
                            for job in jobs {
                                let channel = job.channel.unwrap();
                                let msg = "msg".to_string();
                                let ts = job.msg_ts;
                                let res = Response::new(channel, msg, ts);
                                let send = s
                                    .req_client
                                    .post("https://slack.com/api/chat.postMessage")
                                    .bearer_auth(&s.oauth_token)
                                    .json(&res)
                                    .send()
                                    .await;
                                match send {
                                    Ok(_) => {}
                                    Err(e) => error!("{e}"),
                                }
                            }
                        }
                        Err(e) => error!("bad things happened: {e}"),
                    }
                });
            }
            Err(e) => return report_error(e),
        }

        DELIVER.to_string()
    }

    /// Insert empty film to the database and to the jobs_q
    pub async fn insert_films(&self, message: &str) -> String {
        let (priority, film_names) = message.trim().split_once(' ').unwrap_or_default();

        info!("Inserting {priority} priority films: {film_names:?}");

        let priority = match Priority::from_str(priority) {
            Ok(p) => p,
            Err(_) => return PRI_ERR.to_string(),
        };

        let film_names: Vec<_> = film_names
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        // Concurrently insert all films and insert to jobs_q
        let films: FuturesUnordered<_> = film_names
            .clone()
            .into_iter()
            .map(|film_name| {
                let s = self.state.clone();
                tokio::spawn(async move {
                    match s.db.insert_film(&film_name, priority).await {
                        Ok(f) => match s.queue.insert_job(&f, "").await {
                            Ok(_) => Ok(f),
                            Err(e) => {
                                error!("error inserting to queue: {e}");
                                Err(e)
                            }
                        },
                        Err(e) => {
                            error!("error inserting film to db: {e}");
                            Err(e)
                        }
                    }
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
            Itertools::intersperse(successes.iter(), &", ").for_each(|s| msg += s);
        }

        if film_names.len() - successes.len() > 0 {
            if !successes.is_empty() {
                msg += "\n";
            }
            msg += "Skipped duplicate films:\n";
            Itertools::intersperse(fails, ", ").for_each(|s| msg += s);
        }

        msg
    }
}

fn report_error(e: Error) -> String {
    error!("{e}");
    INTERNAL_ERR.to_string()
}
