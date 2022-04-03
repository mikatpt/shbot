use chrono::Utc;
use color_eyre::Result;
use deadpool_postgres::Runtime::Tokio1;
use models::{Priority, Role};
use serial_test::serial;
use shbot::{
    logger,
    queue::QueueItem,
    store::{Database, PostgresClient},
};
use std::{env, sync::Once};
use tokio::test;

static INIT: Once = Once::new();

/*
Initial setup:
sudo -u postgres psql
CREATE ROLE test WITH PASSWORD 'test';
CREATE DATABASE test_shereebot;
GRANT ALL ON DATABASE test_shereebot TO test;
*/

fn pg_conf() -> deadpool_postgres::Config {
    deadpool_postgres::Config {
        user: Some("test".to_string()),
        password: Some("test".to_string()),
        host: Some("localhost".to_string()),
        port: Some(5432),
        dbname: Some("test_shereebot".to_string()),
        ..Default::default()
    }
}

async fn setup() -> Result<Database<PostgresClient>> {
    INIT.call_once(|| {
        env::set_var("ENVIRONMENT", "test");
        env::set_var("RUST_LOG", "shbot=trace,tower=trace,tower_http=trace");

        dotenv::dotenv().ok();
        logger::install(None);
    });
    let pool = pg_conf().create_pool(Some(Tokio1), tokio_postgres::NoTls)?;

    let client = pool.get().await?;

    let statement = include_str!("../../../schema.sql");
    client.batch_execute(statement).await?;

    let db = Database::<PostgresClient>::new(&pg_conf())?;

    Ok(db)
}

#[test]
#[serial]
async fn films() -> Result<()> {
    let db = setup().await?;

    db.insert_film("b", 0, Priority::High).await?;
    let mut film = db.insert_film("a", 0, Priority::High).await?;
    assert_eq!("a", film.name);

    film.roles.ae = Some("mikatpt".to_string());

    db.update_film(&film).await?;

    let film = db.get_film("a").await?.unwrap();

    assert_eq!(Some("mikatpt".to_string()), film.roles.ae);

    let films = db.list_films().await?;
    assert_eq!(2, films.len());

    Ok(())
}

#[test]
#[serial]
async fn students() -> Result<()> {
    let db = setup().await?;

    // this hits slack api, if it fails say why.
    let id = "U038V25S1MJ";
    let mut student = match db.insert_student(id).await {
        Ok(s) => s,
        Err(_) => panic!("couldn't hit slack api for some reason"),
    };

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

    let films = db.get_student_films(&student.id).await?;
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
