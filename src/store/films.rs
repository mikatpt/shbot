use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio_postgres::Row;
use uuid::Uuid;

use crate::{
    models::{Film, Roles},
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

    pub async fn insert_film(&self, film_name: &str) -> Result<Film> {
        self.client.insert_film(film_name).await
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
            SELECT f.id, f.name, r.ae, r.editor, r.sound, r.color
            FROM films as f, roles as r 
            WHERE f.roles_id = r.id;";
        let stmt = client.prepare_cached(stmt).await?;

        let rows = client.query(&stmt, &[]).await?;
        let films = rows.into_iter().map(format_row_into_film).collect();

        Ok(films)
    }

    async fn get_film(&self, name: &str) -> Result<Option<Film>> {
        let client = self.pool.get().await?;

        let stmt = "
            SELECT f.id, f.name, r.ae, r.editor, r.sound, r.color
            FROM films as f, roles as r 
            WHERE f.name = $1
            AND f.roles_id = r.id;";
        let stmt = client.prepare_cached(stmt).await?;

        let mut rows = client.query(&stmt, &[&name]).await?;
        if rows.is_empty() {
            return Ok(None);
        }
        let row = rows.pop().expect("^^just checked if empty");

        Ok(Some(format_row_into_film(row)))
    }

    async fn insert_film(&self, name: &str) -> Result<Film> {
        let mut client = self.pool.get().await?;

        let stmt = "INSERT INTO roles(id) VALUES($1) RETURNING id;";
        let stmt = client.prepare_cached(stmt).await?;

        let stmt2 = "INSERT INTO films(id, name, roles_id) VALUES($1, $2, $3);";
        let stmt2 = client.prepare_cached(stmt2).await?;

        let transaction = client.transaction().await?;

        // The whole insert should fail if the film already exists.
        let id = Uuid::new_v4();
        let role_id: Uuid = transaction.query(&stmt, &[&id]).await?[0].get("id");

        let mut film = Film::default();
        let id = &film.id;

        let res = transaction.query(&stmt2, &[&id, &name, &role_id]).await;
        if res.is_err() {
            return Err(Error::Duplicate(format!("Film '{name}' already exists")));
        }
        transaction.commit().await?;

        film.name = name.to_string();

        Ok(film)
    }

    async fn update_film(&self, film: &Film) -> Result<()> {
        let client = self.pool.get().await?;

        let stmt = "
            UPDATE roles
            SET ae = $2, editor = $3, sound = $4, color = $5
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
        ]).await?;

        Ok(())
    }
}

// ------------- Helpers ------------- //

fn format_row_into_film(row: Row) -> Film {
    let id: Uuid = row.get("id");
    let name: String = row.get("name");
    let roles: [Option<DateTime<Utc>>; 4] = [
        row.get("ae"),
        row.get("editor"),
        row.get("sound"),
        row.get("color"),
    ];
    let roles = Roles::new(roles[0], roles[1], roles[2], roles[3]);
    Film::new(id, name, roles)
}
