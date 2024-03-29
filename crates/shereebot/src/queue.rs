use std::collections::{BinaryHeap, HashSet};
use std::{cmp::Ordering, sync::Arc};

use chrono::{DateTime, Utc};
use color_eyre::eyre::eyre;
use futures::lock::Mutex;
use tracing::info;
use uuid::Uuid;

use crate::{store::Database, Error, Result};
use models::{Film, Priority, Role, Student};

#[derive(Debug)]
pub(crate) struct Queue {
    pub jobs_q: Q,
    pub wait_q: Q,
    db: Database,
}

type Q = Arc<Mutex<BinaryHeap<QueueItem>>>;

// TODO: we didn't strongly type queue item variants and now it's annoying. maybe fix this?
/// Non-generic queue, works for films and students both.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueItem {
    pub id: Uuid,
    pub student_slack_id: String,
    pub film_name: String,
    pub role: Role,
    pub priority: Option<Priority>,
    pub msg_ts: Option<String>,
    pub channel: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl PartialOrd for QueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueueItem {
    fn cmp(&self, other: &Self) -> Ordering {
        if let (Some(prio), Some(prio_other)) = (self.priority, other.priority) {
            match prio.cmp(&prio_other) {
                Ordering::Equal => {} // If priority is equal, check timestamp.
                ord => return ord,
            }
        }

        match other.created_at.cmp(&self.created_at) {
            Ordering::Equal => {}
            ord => return ord,
        }
        other.film_name.cmp(&self.film_name)
    }
}

impl Queue {
    pub fn _new() -> Self {
        Self {
            jobs_q: Arc::new(Mutex::new(BinaryHeap::new())),
            wait_q: Arc::new(Mutex::new(BinaryHeap::new())),
            db: crate::store::new_mock(),
        }
    }
}

impl Queue {
    pub(crate) async fn from_db(db: Database) -> Result<Self> {
        let wait_q = db.get_queue(true).await?.into_iter().collect();
        let film_q = db.get_queue(false).await?.into_iter().collect();
        Ok(Self {
            jobs_q: Arc::new(Mutex::new(film_q)),
            wait_q: Arc::new(Mutex::new(wait_q)),
            db,
        })
    }

    /// Updates film/student roles and adds film to the jobs_q.
    pub(crate) async fn deliver(&self, mut student: Student, slack_id: &str) -> Result<()> {
        let curr_film = match student.current_film {
            Some(ref f) => f,
            None => return Err(Error::Internal(eyre!("Impossible state"))),
        };

        let mut film = match self.db.get_film(curr_film).await? {
            Some(f) => f,
            None => return Err(Error::Internal(eyre!("Impossible state"))),
        };

        film.increment_role();
        student.increment_role();
        self.db.update_film(&film).await?;
        self.db.update_student(&student).await?;
        self.insert_job(&film, slack_id).await?;

        Ok(())
    }

    /// Returns Vec of jobs.
    /// We must manually attach student slack id to the successes so we can
    /// notify them in the async case.
    pub(crate) async fn try_empty_wait_queue(&self) -> Result<Vec<QueueItem>> {
        let mut wait_q = self.wait_q.lock().await;
        let mut successes = vec![];
        let mut recycle = vec![];

        while let Some(waiter) = wait_q.pop() {
            let id = &waiter.student_slack_id;
            let ts = &waiter.msg_ts.clone().unwrap_or_default();
            let channel = &waiter.channel.clone().unwrap_or_default();
            match self.try_assign_job(id, ts, channel).await {
                Ok(None) => recycle.push(waiter),
                Ok(Some(mut job)) => {
                    job.student_slack_id = id.to_string();
                    successes.push(job)
                }
                Err(e) => return Err(e),
            };
        }
        wait_q.extend(recycle);

        Ok(successes)
    }

    /// Attempt to give a student a job to do. If successful, go ahead and
    /// update the student and film and delete from the db queue.
    pub(crate) async fn try_assign_job(
        &self,
        slack_id: &str,
        ts: &str,
        channel: &str,
    ) -> Result<Option<QueueItem>> {
        let mut student = self.db.get_student(slack_id).await?;
        let group = student.group_number;
        let role = student.current_role;

        if role == Role::Done {
            let e = "You're done! No more work to do :)";
            return Err(Error::Duplicate(e.into()));
        }

        info!("Trying to assign job: now retrieving all films student has worked!");

        // A student should not work the same job twice, unless this is not possible.
        let eligible = self.db.get_films_exclusionary(group, role).await?;
        let eligible: HashSet<_> = eligible.iter().map(|f| f.name.clone()).collect();
        let worked_films = self.db.get_worked_films(&student.id).await?;
        let worked_films: HashSet<_> = worked_films.iter().map(|f| f.name.clone()).collect();

        let mut eligible_films_exist = false;
        for film in &eligible {
            // If student has not worked an eligible film, unique films still exist.
            if !worked_films.contains(film) {
                eligible_films_exist = true;
                break;
            }
        }

        // NOTE:  don't increment until they deliver!
        let role = student.current_role;

        info!("Searching for eligible jobs...");
        let job_to_do = self.get_job(worked_films, role, eligible_films_exist).await;

        // If there was no suitable job found, insert student into the wait queue
        if job_to_do.is_none() {
            info!("No job found - inserting {} to the wait_q", &student.name);
            self.insert_waiter(role, ts, channel, &student.slack_id)
                .await?;
            return Ok(None);
        }

        info!("Updating student and film records and removing job from queue");

        // ...otherwise, remove job from db and add a students_films record.
        // Also, update the student and film records to reflect the current state.
        let job = job_to_do.unwrap();
        match self.db.get_film(&job.film_name).await? {
            Some(film) => {
                student.current_film = Some(film.name);
                self.db.insert_student_films(&student.id, &film.id).await?;
                self.db.update_student(&student).await?;
                self.db.delete_from_queue(&job.id, false).await?;
            }
            None => return Err(Error::Internal(eyre!("Impossible state"))),
        }

        info!("Assigned {} to {}", student.name, job.film_name);
        Ok(Some(job))
    }

    async fn get_job(
        &self,
        worked_films: HashSet<String>,
        role: Role,
        eligible_films_exist: bool,
    ) -> Option<QueueItem> {
        let mut work_q = self.jobs_q.lock().await;
        let mut recycle = vec![];

        let mut eligible_job: Option<QueueItem> = None;

        // Search queue for a job to work on, recycle entries that don't fit.
        while let Some(job) = work_q.pop() {
            let worked_on_film = worked_films.contains(&job.film_name);

            let eligible = job.role == role;

            // Only assign if role is correct.
            // Then, only assign if student hasn't worked on the film before,
            // BUT assign anyways if there are no unique films left.
            if eligible && (!worked_on_film || !eligible_films_exist) {
                eligible_job = Some(job);
                break;
            } else {
                recycle.push(job);
            }
        }
        work_q.extend(recycle);

        eligible_job
    }

    pub(crate) async fn insert_job(&self, f: &Film, slack_id: &str) -> Result<QueueItem> {
        let mut jobs_q = self.jobs_q.lock().await;

        let job = QueueItem {
            id: Uuid::new_v4(),
            student_slack_id: slack_id.to_string(),
            film_name: f.name.clone(),
            channel: None,
            msg_ts: None,
            priority: Some(f.priority),
            created_at: Utc::now(),
            role: f.current_role,
        };

        jobs_q.push(job.clone());
        self.db.insert_to_queue(job, false).await
    }

    async fn insert_waiter(
        &self,
        role: Role,
        msg_ts: &str,
        channel: &str,
        slack_id: &str,
    ) -> Result<QueueItem> {
        let mut wait_q = self.wait_q.lock().await;

        let waiter = QueueItem {
            id: Uuid::new_v4(),
            student_slack_id: slack_id.to_string(),
            film_name: "".to_string(),
            channel: Some(channel.to_string()),
            msg_ts: Some(msg_ts.to_string()),
            priority: None,
            created_at: Utc::now(),
            role,
        };
        wait_q.push(waiter.clone());
        self.db.insert_to_queue(waiter, true).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Duration, Utc};

    #[test]
    // Queue should pop high priority items, then earliest items, then alphabetical.
    fn check_order() {
        let yesterday = Utc::now() - Duration::days(1);
        let today = Utc::now();

        let mut jobs = vec![
            get_job("b", Priority::Low, today),
            get_job("a", Priority::Low, today),
            get_job("a", Priority::Low, yesterday),
            get_job("b", Priority::High, today),
            get_job("a", Priority::High, today),
            get_job("b", Priority::High, yesterday),
            get_job("a", Priority::High, yesterday),
        ];

        let mut job_queue: BinaryHeap<QueueItem> = jobs.clone().into_iter().collect();

        job_queue.iter().for_each(|j| println!("{j:?}"));

        while let (Some(expected), Some(actual)) = (jobs.pop(), job_queue.pop()) {
            assert_eq!(expected, actual);
        }
    }

    fn get_job(name: &str, priority: Priority, date: DateTime<Utc>) -> QueueItem {
        QueueItem {
            id: Uuid::new_v4(),
            film_name: name.to_string(),
            priority: Some(priority),
            created_at: date,
            student_slack_id: "".to_string(),
            role: Role::Ae,
            msg_ts: None,
            channel: None,
        }
    }
}
