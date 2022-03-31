use std::cmp::Ordering;
use std::collections::BinaryHeap;

use chrono::{DateTime, Utc};

use crate::models::{Priority, Role};

/// All films ready to be picked up.
#[derive(Debug)]
pub struct FilmQ {}

/// Non-generic queue, works for films and students both.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Queue {
    student_slack_id: String,
    film_name: String,
    role: Role,
    priority: Option<Priority>,
    created_at: DateTime<Utc>,
}

impl PartialOrd for Queue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Queue {
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

        let mut job_queue: BinaryHeap<Queue> = jobs.clone().into_iter().collect();

        while let (Some(expected), Some(actual)) = (jobs.pop(), job_queue.pop()) {
            assert_eq!(expected, actual);
        }
    }

    fn get_job(name: &str, priority: Priority, date: DateTime<Utc>) -> Queue {
        Queue {
            film_name: name.to_string(),
            priority: Some(priority),
            created_at: date,
            student_slack_id: "".to_string(),
            role: Role::Ae,
        }
    }
}
