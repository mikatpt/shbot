use std::collections::BinaryHeap;
use std::{cmp::Ordering, sync::Arc};

use chrono::{DateTime, Utc};
use futures::lock::Mutex;

use crate::{
    models::{Priority, Role},
    store::{Client, Database},
    Result,
};

#[derive(Debug, Default)]
pub struct Queue {
    pub film_q: Q,
    pub wait_q: Q,
}

impl Queue {
    pub(crate) async fn from_db<T: Client>(db: Database<T>) -> Result<Queue> {
        let wait_q = db.get_queue(true).await?.into_iter().collect();
        let film_q = db.get_queue(false).await?.into_iter().collect();
        Ok(Self {
            film_q: Arc::new(Mutex::new(film_q)),
            wait_q: Arc::new(Mutex::new(wait_q)),
        })
    }
}

type Q = Arc<Mutex<BinaryHeap<QueueItem>>>;

/// Non-generic queue, works for films and students both.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueItem {
    pub student_slack_id: String,
    pub film_name: String,
    pub role: Role,
    pub priority: Option<Priority>,
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

        while let (Some(expected), Some(actual)) = (jobs.pop(), job_queue.pop()) {
            assert_eq!(expected, actual);
        }
    }

    fn get_job(name: &str, priority: Priority, date: DateTime<Utc>) -> QueueItem {
        QueueItem {
            film_name: name.to_string(),
            priority: Some(priority),
            created_at: date,
            student_slack_id: "".to_string(),
            role: Role::Ae,
        }
    }
}
