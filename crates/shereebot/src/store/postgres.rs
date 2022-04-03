#![allow(dead_code)]
use std::{collections::HashSet, str::FromStr};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use color_eyre::eyre::eyre;
use deadpool_postgres::Pool;
use tokio_postgres::Row;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{
    models::{Film, Priority, Role, Roles, Student},
    queue::QueueItem,
    slack::UserResponse,
    store::Client,
    Error, Result,
};

/// Internal Postgres client.
pub struct PostgresClient {
    pool: Pool,
}

impl PostgresClient {
    pub(crate) fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Client for PostgresClient {
    // ------------- Films ------------- //

    async fn list_films(&self) -> Result<Vec<Film>> {
        info!("Retrieving all films");
        let client = self.pool.get().await?;

        let stmt = "
            SELECT f.id, f.name, f.priority, r.ae, r.editor, r.sound, r.color, r.current
            FROM films as f, roles as r 
            WHERE f.roles_id = r.id;";
        let stmt = client.prepare_cached(stmt).await?;

        let rows = client.query(&stmt, &[]).await?;
        let films: Result<Vec<_>> = rows.into_iter().map(format_row_into_film).collect();

        Ok(films?)
    }

    async fn get_film(&self, name: &str) -> Result<Option<Film>> {
        let client = self.pool.get().await?;

        let stmt = "
            SELECT f.id, f.name, f.priority, r.ae, r.editor, r.sound, r.color, r.current
            FROM films as f, roles as r 
            WHERE f.name = $1
            AND f.roles_id = r.id;";
        let stmt = client.prepare_cached(stmt).await?;

        let mut rows = client.query(&stmt, &[&name]).await?;
        if rows.is_empty() {
            return Ok(None);
        }
        let row = rows.pop().expect("^^just checked if empty");

        let film = Some(format_row_into_film(row)?);
        info!("Retrieved film {name}");

        Ok(film)
    }

    async fn insert_film(&self, name: &str, priority: Priority) -> Result<Film> {
        let mut client = self.pool.get().await?;

        let stmt = "INSERT INTO roles(id) VALUES($1) RETURNING id;";
        let stmt = client.prepare_cached(stmt).await?;

        let stmt2 = "INSERT INTO films(id, name, priority, roles_id) VALUES($1, $2, $3, $4);";
        let stmt2 = client.prepare_cached(stmt2).await?;

        let transaction = client.transaction().await?;

        // The whole insert should fail if the film already exists.
        let id = Uuid::new_v4();
        let role_id: Uuid = transaction.query(&stmt, &[&id]).await?[0].get("id");

        let mut film = Film::default();
        let id = &film.id;
        let p = priority.as_ref();

        let res = transaction.query(&stmt2, &[&id, &name, &p, &role_id]).await;
        if res.is_err() {
            return Err(Error::Duplicate(name.to_string()));
        }
        transaction.commit().await?;

        info!("Inserted film: {}", name);
        film.name = name.to_string();

        Ok(film)
    }

    async fn update_film(&self, film: &Film) -> Result<()> {
        let client = self.pool.get().await?;

        let stmt = "
            UPDATE roles
            SET ae = $2, editor = $3, sound = $4, color = $5, current = $6
            WHERE id = (
                SELECT roles_id FROM films WHERE name = $1);";
        let stmt = client.prepare_cached(stmt).await?;

        #[rustfmt::skip]
        client.query(&stmt, &[
            &film.name,
            &film.roles.ae,
            &film.roles.editor,
            &film.roles.sound,
            &film.roles.color,
            &film.current_role.as_ref(),
        ]).await?;

        info!("Updated film: {}", film.name);

        Ok(())
    }

    // ------------- Junction ------------- //

    async fn get_student_films(&self, student_id: &Uuid) -> Result<HashSet<Film>> {
        let client = self.pool.get().await?;

        let stmt = "
            SELECT f.id, f.name, f.priority, r.ae, r.editor, r.sound, r.color, r.current
            FROM films as f 
                JOIN roles AS r ON f.roles_id = r.id 
                JOIN students_films on f.id = students_films.film_id
            WHERE students_films.student_id = $1;";
        let stmt = client.prepare_cached(stmt).await?;

        let rows = client.query(&stmt, &[&student_id]).await?;

        let res: Result<HashSet<_>> = rows.into_iter().map(format_row_into_film).collect();
        let res = res?;
        info!("Retrieved {} rows from students_films", res.len());

        Ok(res)
    }

    async fn insert_student_films(&self, student_id: &Uuid, film_id: &Uuid) -> Result<()> {
        let client = self.pool.get().await?;

        let stmt = "INSERT INTO students_films(student_id, film_id) VALUES($1, $2);";
        let stmt = client.prepare_cached(stmt).await?;

        client.query(&stmt, &[&student_id, &film_id]).await?;
        info!("Inserted into students_films");

        Ok(())
    }

    // ------------- Students ------------- //

    async fn list_students(&self) -> Result<Vec<Student>> {
        info!("Retrieving list of students");
        let client = self.pool.get().await?;

        let stmt = "
            SELECT s.id, s.name, s.slack_id, s.current_film, 
                   r.ae, r.editor, r.sound, r.color, r.current
            FROM students as s, roles as r 
            WHERE s.roles_id = r.id;";
        let stmt = client.prepare_cached(stmt).await?;

        let rows = client.query(&stmt, &[]).await?;
        let students: Result<Vec<_>> = rows.into_iter().map(format_row_into_student).collect();

        Ok(students?)
    }

    async fn get_student(&self, slack_id: &str) -> Result<Student> {
        debug!("Retrieving student with id {slack_id}");
        let client = self.pool.get().await?;

        let stmt = "
            SELECT s.id, s.name, s.slack_id, s.current_film, 
                   r.ae, r.editor, r.sound, r.color, r.current
            FROM students as s, roles as r 
            WHERE s.slack_id = $1
            AND s.roles_id = r.id;";
        let stmt = client.prepare_cached(stmt).await?;

        let mut rows = client.query(&stmt, &[&slack_id]).await?;
        if rows.is_empty() {
            warn!("No user for id {slack_id}. Inserting instead.");
            return self.insert_student(slack_id).await;
        }
        let row = rows.pop().expect("^^just checked if empty");

        let student = format_row_into_student(row)?;
        info!("Retrieved student {}", student.name);

        Ok(student)
    }

    async fn insert_student(&self, slack_id: &str) -> Result<Student> {
        // Get student name from slack api
        let client = reqwest::Client::builder().build()?;
        let token = std::env::var("OAUTH_TOKEN").map_err(Into::<Error>::into)?;
        let req = format!("https://slack.com/api/users.info?user={slack_id}");

        let res = client.post(req).bearer_auth(token).send().await?;
        let res = res.json::<UserResponse>().await?;

        let name = res.user.real_name;

        // Insert student
        let mut client = self.pool.get().await?;

        let stmt = "INSERT INTO roles(id) VALUES($1) RETURNING id;";
        let stmt = client.prepare_cached(stmt).await?;

        let stmt2 = "INSERT INTO students(id, name, roles_id, slack_id) VALUES($1, $2, $3, $4);";
        let stmt2 = client.prepare_cached(stmt2).await?;

        let transaction = client.transaction().await?;

        // The whole insert should fail if the student already exists.
        let role_id = Uuid::new_v4();
        transaction.query(&stmt, &[&role_id]).await?;

        let mut student = Student::default();
        let id = &student.id;

        let res = transaction
            .query(&stmt2, &[&id, &name, &role_id, &slack_id])
            .await;

        if res.is_err() {
            return Err(Error::Duplicate(name.to_string()));
        }
        transaction.commit().await?;

        info!("Inserted student: {}", name);
        student.slack_id = slack_id.to_string();
        student.name = name;

        Ok(student)
    }

    async fn update_student(&self, student: &Student) -> Result<()> {
        let mut client = self.pool.get().await?;

        let stmt = "
            UPDATE roles
            SET ae = $2, editor = $3, sound = $4, color = $5, current = $6
            WHERE id = (
                SELECT roles_id FROM students WHERE id = $1);";
        let stmt = client.prepare_cached(stmt).await?;

        let stmt2 = "UPDATE students SET current_film = $2 WHERE id = $1";
        let stmt2 = client.prepare_cached(stmt2).await?;

        let transaction = client.transaction().await?;

        #[rustfmt::skip]
        transaction.query(&stmt, &[
            &student.id,
            &student.roles.ae,
            &student.roles.editor,
            &student.roles.sound,
            &student.roles.color,
            &student.current_role.as_ref(),
        ]).await?;

        transaction
            .query(&stmt2, &[&student.id, &student.current_film])
            .await?;

        transaction.commit().await?;

        info!("Updated student: {}", student.name);

        Ok(())
    }

    // ------------- Queue ------------- //

    async fn get_queue(&self, wait: bool) -> Result<Vec<QueueItem>> {
        let client = self.pool.get().await?;

        let stmt = if wait {
            "SELECT * from wait_q;"
        } else {
            "SELECT * from jobs_q;"
        };

        let stmt = client.prepare_cached(stmt).await?;

        let rows = client.query(&stmt, &[]).await?;
        let mut res = vec![];
        for row in rows {
            let id: Uuid = row.get("id");
            let student_slack_id: String = row.get("student_slack_id");
            let film_name: String = row.get("film_name");
            let role = Role::from_str(row.get("role"))?;
            let p: Option<String> = row.get("priority");
            let priority = match p {
                Some(pr) => Some(Priority::from_str(&pr)?),
                None => None,
            };
            let msg_ts: Option<String> = row.get("msg_ts");
            let channel: Option<String> = row.get("channel");
            let created_at: DateTime<Utc> = row.get("created_at");

            #[rustfmt::skip]
            let item = QueueItem {
                id, student_slack_id, film_name, role,
                priority, created_at, msg_ts, channel,
            };
            res.push(item);
        }

        let s = if wait { "wait" } else { "jobs" };
        info!("Retrieved {s} queue");

        Ok(res)
    }

    async fn insert_to_queue(&self, q: QueueItem, wait: bool) -> Result<QueueItem> {
        let client = self.pool.get().await?;

        if wait {
            let stmt = "INSERT INTO wait_q(id, student_slack_id, film_name, role, msg_ts, channel)
             VALUES($1, $2, $3, $4, $5, $6);";
            let stmt = client.prepare_cached(stmt).await?;

            #[rustfmt::skip]
            client.query(&stmt, &[
                &q.id,
                &q.student_slack_id,
                &q.film_name,
                &q.role.as_ref(),
                &q.msg_ts,
                &q.channel,
            ]).await?;
        } else {
            let stmt = "INSERT INTO jobs_q(id, student_slack_id, film_name, role, priority)
             VALUES($1, $2, $3, $4, $5);";
            let stmt = client.prepare_cached(stmt).await?;
            let p = q.priority.map(|a| a.as_ref().to_string());

            #[rustfmt::skip]
            client.query(&stmt, &[
                &q.id,
                &q.student_slack_id,
                &q.film_name,
                &q.role.as_ref(),
                &p,
            ]).await?;
        }

        if wait {
            info!("Inserted {} into the wait queue", q.student_slack_id);
        } else {
            info!("Inserted {} into the jobs queue", q.film_name);
        }

        Ok(q)
    }

    async fn delete_from_queue(&self, id: &Uuid, wait: bool) -> Result<()> {
        let client = self.pool.get().await?;

        let stmt = if wait {
            "DELETE FROM wait_q WHERE id = $1;"
        } else {
            "DELETE FROM jobs_q WHERE id = $1;"
        };
        let stmt = client.prepare_cached(stmt).await?;

        client.query(&stmt, &[&id]).await?;

        Ok(())
    }

    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }

    async fn drop_db(&self) -> Result<()> {
        let environment = std::env::var("ENVIRONMENT")?;
        if environment != "test" {
            return Err(Error::Internal(eyre!("Cannot drop db outside of testing")));
        }

        let db = std::env::var("TEST_POSTGRES_DBNAME")?;
        let client = self.pool.get().await?;
        let stmt = client
            .prepare_cached(&format!("DROP DATABASE {};", db))
            .await?;
        client.query(&stmt, &[]).await?;

        Ok(())
    }
}

// ------------- Helpers ------------- //

fn format_row_into_film(row: Row) -> Result<Film> {
    let id: Uuid = row.get("id");
    let name: String = row.get("name");
    let priority = Priority::from_str(row.get("priority"))?;
    let role = Role::from_str(row.get("current"))?;

    let roles: [Option<DateTime<Utc>>; 4] = [
        row.get("ae"),
        row.get("editor"),
        row.get("sound"),
        row.get("color"),
    ];
    let roles = Roles::new(roles[0], roles[1], roles[2], roles[3]);
    Ok(Film::new(id, name, role, priority, roles))
}

fn format_row_into_student(row: Row) -> Result<Student> {
    let id: Uuid = row.get("id");
    let name: String = row.get("name");
    let slack_id: String = row.get("slack_id");
    let current_film: Option<String> = row.get("current_film");
    let current_role = Role::from_str(row.get("current"))?;

    let roles: [Option<DateTime<Utc>>; 4] = [
        row.get("ae"),
        row.get("editor"),
        row.get("sound"),
        row.get("color"),
    ];
    let roles = Roles::new(roles[0], roles[1], roles[2], roles[3]);

    #[rustfmt::skip]
    let student = Student { id, name, slack_id, current_film, current_role, roles };

    Ok(student)
}
