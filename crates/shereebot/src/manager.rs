#![allow(dead_code)]
use std::collections::HashSet;

use csv_parser::{FilmInput, StudentInput};
use futures::{future, stream::FuturesUnordered};
use itertools::Itertools;
use tracing::{debug, error, info, trace};

use crate::{
    queue::QueueItem,
    server::State,
    slack::{app_mentions::Response, events::File},
    store::Client,
    Error, Result,
};
use models::{Film, Priority, Student};

pub(crate) struct Manager<T: Client> {
    state: State<T>,
}

const DELIVER: &str = "Good job!! You've delivered your work.

When you're ready to pick up another job, just type `@ShereeBot request-work`.
Then, I'll message you back when there's a job ready for you.";

const INTERNAL_ERR: &str = "Something went wrong internally - please let Sheree know!";

// const PRI_ERR: &str = "I couldn't read your command :cry:
// Valid priority weights are `HIGH` and `LOW`.
// Ex: `insert-films HIGH film1, film2, film3`";

const NO_WORK: &str = "No work is available yet :cry:
I'll reply in this thread once I find some work for you!";

impl<T: Client> Manager<T> {
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

    /// Insert one film to the database.
    pub async fn insert_film(
        &self,
        film_name: &str,
        group: i32,
        priority: Priority,
    ) -> Result<Film> {
        insert_film(self.state.clone(), film_name, group, priority).await
    }

    /// Insert empty film to the database and to the jobs_q
    #[tracing::instrument(skip(self))]
    pub async fn insert_films(&self, films: Vec<Film>) -> String {
        info!("Inserting films");

        // Concurrently insert all films and insert to jobs_q
        let films_fut: FuturesUnordered<_> = films
            .clone()
            .into_iter()
            .map(|f| {
                let s = self.state.clone();
                tokio::spawn(
                    async move { insert_film(s, &f.name, f.group_number, f.priority).await },
                )
            })
            .collect();

        // Await all inserts to complete.
        let v: Result<Vec<_>> = future::join_all(films_fut)
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
            .map(|r| r.as_ref().unwrap().name.as_str())
            .collect();

        let fails = films
            .iter()
            .filter(|&n| !successes.contains(n.name.as_str()))
            .map(|s| s.name.as_ref());

        let mut msg = String::new();

        if !successes.is_empty() {
            msg += &format!("Successfully inserted {} films(s)!\n", successes.len());
            Itertools::intersperse(successes.iter(), &", ").for_each(|s| msg += s);
        }

        if films.len() - successes.len() > 0 {
            if !successes.is_empty() {
                msg += "\n";
            }
            msg += "Skipped duplicate films:\n";
            Itertools::intersperse(fails, ", ").for_each(|s| msg += s);
        }

        msg
    }

    /// Bulk insert from csv file.
    pub async fn insert_students_from_csv(&self, students: Vec<Student>) -> String {
        info!("Inserting students");

        // Concurrently insert all students
        let students_fut: FuturesUnordered<_> = students
            .clone()
            .into_iter()
            .map(|st| {
                let s = self.state.clone();
                let (name, group, class) = (st.name, st.group_number, st.class);
                tokio::spawn(
                    async move { s.db.insert_student_from_csv(&name, group, &class).await },
                )
            })
            .collect();

        // Await all inserts to complete.
        let students_fut: Result<Vec<_>> = future::join_all(students_fut)
            .await
            .into_iter()
            .map(|r| r.map_err(Into::<Error>::into))
            .collect();

        let students_fut = match students_fut {
            Ok(r) => r,
            Err(e) => return e.to_string(),
        };

        let successes: HashSet<_> = students_fut
            .iter()
            .filter(|r| r.is_ok())
            .map(|r| r.as_ref().unwrap().name.as_str())
            .collect();

        let fails = students
            .iter()
            .filter(|&n| !successes.contains(n.name.as_str()))
            .map(|s| s.name.as_ref());

        let mut msg = String::new();

        if !successes.is_empty() {
            msg += &format!("Successfully inserted {} student(s)!\n", successes.len());
            Itertools::intersperse(successes.iter(), &", ").for_each(|s| msg += s);
        }

        if students.len() - successes.len() > 0 {
            if !successes.is_empty() {
                msg += "\n";
            }
            msg += "Skipped duplicate students:\n";
            Itertools::intersperse(fails, ", ").for_each(|s| msg += s);
        }

        msg
    }

    pub async fn insert_from_files(&self, files: &[File]) -> Result<String> {
        let mut messages: Vec<String> = vec![];

        for file in files {
            trace!("file: {file:?}");

            if file.name.contains("film") {
                info!("downloading films csv into db");
                let v: Vec<Film> = csv_parser::from_url::<FilmInput>(&file.url_private_download)
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect();

                messages.push(self.insert_films(v).await);
            } else if file.name.contains("student") {
                info!("downloading students csv into db");
                let v: Vec<Student> =
                    csv_parser::from_url::<StudentInput>(&file.url_private_download)
                        .await?
                        .into_iter()
                        .map(Into::into)
                        .collect();

                messages.push(self.insert_students_from_csv(v).await)
            }
        }

        Ok(messages.into_iter().map(|m| m + "\n").collect())
    }
}

async fn insert_film<T: Client>(
    state: State<T>,
    film_name: &str,
    group: i32,
    priority: Priority,
) -> Result<Film> {
    match state.db.insert_film(film_name, group, priority).await {
        Ok(f) => match state.queue.insert_job(&f, "").await {
            Ok(_) => Ok(f),
            Err(e) => {
                error!("Error inserting to queue: {e}");
                Err(e)
            }
        },
        Err(e) => {
            error!("error inserting film to db");
            Err(e)
        }
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
