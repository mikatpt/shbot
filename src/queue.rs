use std::collections::{BinaryHeap, HashSet};
use std::{cmp::Ordering, sync::Arc};

use chrono::{DateTime, Utc};
use color_eyre::eyre::eyre;
use futures::lock::Mutex;
use uuid::Uuid;

use crate::models::Student;
use crate::store::mock::MockClient;
use crate::{
    models::{Film, Priority, Role},
    store::{Client, Database},
    Error, Result,
};

#[derive(Debug)]
pub(crate) struct Queue<T: Client> {
    pub jobs_q: Q,
    pub wait_q: Q,
    db: Database<T>,
}

impl Queue<MockClient> {
    pub fn _new() -> Self {
        Self {
            jobs_q: Arc::new(Mutex::new(BinaryHeap::new())),
            wait_q: Arc::new(Mutex::new(BinaryHeap::new())),
            db: Database::<MockClient>::new(),
        }
    }
}

impl<T: Client> Queue<T> {
    pub(crate) async fn from_db(db: Database<T>) -> Result<Self> {
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
                Ok(Some(job)) => successes.push(job),
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

        if student.current_role == Role::Done {
            let e = "You're done! No more work to do :)";
            return Err(Error::Duplicate(e.into()));
        }
        let student_films = self.db.get_student_films(&student.id).await?;
        let worked_films: HashSet<_> = student_films.iter().map(|f| f.name.clone()).collect();

        // NOTE:  don't increment until they deliver!
        let role = student.current_role;

        let job_to_do = self.get_job(worked_films, slack_id, role).await;

        // If there was no suitable job found, insert student into the wait queue
        if job_to_do.is_none() {
            self.insert_waiter(role, ts, channel, &student.slack_id)
                .await?;
            return Ok(None);
        }

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

        Ok(Some(job))
    }

    async fn get_job(
        &self,
        worked_films: HashSet<String>,
        slack_id: &str,
        role: Role,
    ) -> Option<QueueItem> {
        let mut work_q = self.jobs_q.lock().await;
        let mut recycle = vec![];

        let mut eligible_job: Option<QueueItem> = None;

        // Search queue for a job to work on, recycle entries that don't fit.
        while let Some(job) = work_q.pop() {
            // I think this is redundant but we'll stay safe for now.
            let same_id = job.student_slack_id == slack_id;
            let worked_on_film = worked_films.contains(&job.film_name);

            let worked_on = same_id || worked_on_film;
            let eligible = job.role == role;

            if eligible && !worked_on {
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

type Q = Arc<Mutex<BinaryHeap<QueueItem>>>;

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
