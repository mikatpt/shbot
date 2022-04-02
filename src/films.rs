#![allow(dead_code)]
use std::{collections::HashSet, str::FromStr};

use futures::{future, stream::FuturesUnordered};
use itertools::Itertools;
use tracing::{debug, error, info};

use crate::{
    models::Priority, queue::QueueItem, server::State, slack::app_mentions::Response,
    store::Client, Error, Result,
};

pub(crate) struct FilmManager<T: Client> {
    state: State<T>,
}

const DELIVER: &str = "Good job!! You've delivered your work.

When you're ready to pick up another job, just type `@ShereeBot request-work`.
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
    #[tracing::instrument(skip(self, ts, channel))]
    pub async fn request_work(&self, slack_id: &str, ts: &str, channel: &str) -> String {
        match self.state.queue.try_assign_job(slack_id, ts, channel).await {
            Ok(Some(j)) => {
                format!(
                    "<@{}> You've been assigned to work `{}` on `{}`!",
                    slack_id,
                    j.role.as_ref(),
                    j.film_name
                )
            }
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
    #[tracing::instrument(skip(self))]
    pub async fn deliver_work(&self, slack_id: &str) -> String {
        debug!("Delivering work for {slack_id}");
        let student = match self.state.db.get_student(slack_id).await {
            Ok(u) => u,
            Err(e) => return report_error(e),
        };
        let name = student.name.clone();
        match self.state.queue.deliver(student, slack_id).await {
            Ok(_) => self.empty_wait_queue().await,
            Err(e) => return report_error(e),
        }

        info!("Successful delivery for {}!", name);

        DELIVER.to_string()
    }

    /// After delivering the work, we'll try to assign jobs out to the wait queue.
    /// This is done in the background via a tokio task.
    async fn empty_wait_queue(&self) {
        let s = self.state.clone();
        tokio::spawn(async move {
            let (client, token) = (&s.req_client, &s.oauth_token);
            match s.queue.try_empty_wait_queue().await {
                Ok(jobs) => {
                    for job in jobs {
                        notify_waiter(client, token, job).await;
                    }
                }
                Err(e) => error!("bad things happened: {e}"),
            }
        });
    }

    /// Insert empty film to the database and to the jobs_q
    #[tracing::instrument(skip(self))]
    pub async fn insert_films(&self, message: &str) -> String {
        let (priority, film_names) = message.trim().split_once(' ').unwrap_or_default();

        info!("Inserting {priority} priority films");

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

/// NOTE: You must manually attach a student slack id to the job when calling this!
async fn notify_waiter(client: &reqwest::Client, token: &str, job: QueueItem) {
    info!("Notifying waiter: assigned out {}", job.film_name);

    let msg = format!(
        "<@{}> You've been assigned to work `{}` on `{}`!",
        job.student_slack_id,
        job.role.as_ref(),
        job.film_name
    );
    let res = Response::new(job.channel.unwrap(), msg, job.msg_ts);
    let send = client
        .post("https://slack.com/api/chat.postMessage")
        .bearer_auth(token)
        .json(&res)
        .send()
        .await;

    if let Err(e) = send {
        error!("{e}");
    }
}

fn report_error(e: Error) -> String {
    error!("{e}");
    INTERNAL_ERR.to_string()
}
