use std::{env, process::Command, sync::Once};

use chrono::Utc;
use color_eyre::{Help, Result};
use deadpool_postgres::Runtime::Tokio1;
use models::{Priority, Role};
use serial_test::serial;
use shbot::{
    logger,
    queue::QueueItem,
    store::{Database, PostgresClient},
};
use tokio::test;
use tracing::info;

static INIT: Once = Once::new();

fn pg_conf() -> deadpool_postgres::Config {
    deadpool_postgres::Config {
        user: Some("test".to_string()),
        password: Some("test".to_string()),
        host: Some("0".to_string()),
        port: Some(5435),
        dbname: Some("shereebot".to_string()),
        ..Default::default()
    }
}

async fn setup() -> Result<Database<PostgresClient>> {
    INIT.call_once(|| {
        env::set_var("ENVIRONMENT", "test");
        // set to trace to debug things
        env::set_var("RUST_LOG", "error");

        let file = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/test.docker-compose.yml");

        // Start the docker test container.
        let output = Command::new("docker")
            .args(["compose", "-f", file, "up", "-d"])
            .output()
            .expect("Failed to run command");

        if !output.status.success() {
            panic!("failed to spin up docker container");
        }

        dotenv::dotenv().ok();
        logger::install(None);
        info!("Started up docker!");
    });
    let pool = pg_conf().create_pool(Some(Tokio1), tokio_postgres::NoTls)?;

    let client = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            return Err(e)
                .with_warning(|| "Docker didn't start fast enough.")
                .with_suggestion(|| "Try running `cargo test` again.");
        }
    };

    let statement = include_str!("./test_schema.sql");
    client.batch_execute(statement).await?;

    let db = Database::<PostgresClient>::new(&pg_conf())?;

    Ok(db)
}

#[test]
#[serial]
async fn films() -> Result<()> {
    let db = setup().await?;

    db.insert_film("b", 1, Priority::High).await?;
    let mut film = db.insert_film("a", 1, Priority::High).await?;
    assert_eq!("a", film.name);

    film.roles.ae = Some("mikatpt".to_string());

    db.update_film(&film).await?;

    let film = db.get_film("a").await?.unwrap();

    assert_eq!(Some("mikatpt".to_string()), film.roles.ae);

    let films = db.list_films().await?;
    assert_eq!(2, films.len());

    Ok(())
}

#[tokio::test]
#[serial]
async fn students() -> Result<()> {
    let db = setup().await?;

    // this hits slack api, if it fails say why.
    let id = "U038V25S1MJ";
    let mut student = db.insert_student(id, "a").await?;

    student.roles.ae = Some("star wars".to_string());

    db.update_student(&student).await?;

    let student = db.get_student(id).await?;

    assert_eq!(Some("star wars".to_string()), student.roles.ae);

    let sts = db.list_students().await?;
    assert_eq!(1, sts.len());

    let film = db.insert_film("a", 0, Priority::High).await?;
    let film2 = db.insert_film("b", 0, Priority::High).await?;

    db.insert_student_films(&student.id, &film.id).await?;
    db.insert_student_films(&student.id, &film2.id).await?;

    let films = db.get_worked_films(&student.id).await?;
    assert!(films.contains(&film));
    assert!(films.contains(&film2));

    Ok(())
}

#[test]
#[serial]
async fn queue() -> Result<()> {
    let db = setup().await?;

    let date_str = "Tue, 1 Jul 2003 10:52:37 +0200";
    let date = chrono::DateTime::parse_from_rfc2822(date_str).unwrap();
    let date = date.with_timezone(&Utc);

    let job_q = QueueItem {
        id: uuid::Uuid::new_v4(),
        student_slack_id: "U038V25S1MJ".to_string(),
        film_name: "test".to_string(),
        role: Role::Ae,
        priority: Some(Priority::High),
        msg_ts: None,
        channel: None,
        created_at: date,
    };

    let mut wait_q = job_q.clone();
    wait_q.id = uuid::Uuid::new_v4();
    wait_q.priority = None;
    wait_q.msg_ts = Some("1234".to_string());
    wait_q.channel = Some("ASD".to_string());

    db.insert_to_queue(wait_q.clone(), true).await?;
    db.insert_to_queue(job_q.clone(), false).await?;
    db.delete_from_queue(&wait_q.id, true).await?;
    db.delete_from_queue(&job_q.id, true).await?;

    let mut jobs = db.get_queue(false).await?;
    let mut j = jobs.pop().unwrap();
    j.created_at = job_q.created_at;
    assert_eq!(j, job_q);

    let waits = db.get_queue(true).await?;
    assert!(waits.is_empty());

    Ok(())
}
