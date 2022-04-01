#![allow(dead_code)]
use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio_postgres::Row;
use tracing::info;
use uuid::Uuid;

use crate::{
    models::{Film, Priority, Role, Roles},
    store::{Client, Database, PostgresClient},
    Error, Result,
};

/// Server-facing API implementation for films.
impl Database<PostgresClient> {
    pub async fn list_films(&self) -> Result<Vec<Film>> {
        self.client.list_films().await
    }

    pub async fn get_film(&self, film_name: &str) -> Result<Option<Film>> {
        self.client.get_film(film_name).await
    }

    pub async fn insert_film(&self, name: &str, priority: Priority) -> Result<Film> {
        self.client.insert_film(name, priority).await
    }

    pub async fn update_film(&self, film: &Film) -> Result<()> {
        self.client.update_film(film).await
    }
}

#[async_trait]
impl Client for PostgresClient {
    async fn list_films(&self) -> Result<Vec<Film>> {
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

        Ok(Some(format_row_into_film(row)?))
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
