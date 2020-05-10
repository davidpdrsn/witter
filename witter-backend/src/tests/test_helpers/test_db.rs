use sqlx::prelude::Connect;
use sqlx::PgConnection;
use sqlx::PgPool;
use sqlx::Postgres;
use std::env;

#[derive(Debug)]
pub struct TestDb {
    db_url: String,
    db_pool: Option<PgPool>,
}

/// Sets up a new DB for running tests with.
impl TestDb {
    pub async fn new() -> Self {
        dotenv::dotenv().ok();
        pretty_env_logger::try_init().ok();

        let db_url = db_url();
        create_db(&db_url).await;
        run_migrations(&db_url).await;

        let db_pool = PgPool::new(&db_url).await.unwrap();

        Self {
            db_url,
            db_pool: Some(db_pool),
        }
    }

    pub fn db(&self) -> PgPool {
        self.db_pool.clone().unwrap()
    }
}

impl Drop for TestDb {
    fn drop(&mut self) {
        // Drop the DB Pool
        let _ = self.db_pool.take();
        futures::executor::block_on(drop_db(&self.db_url));
    }
}

fn db_url() -> String {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    // Set up the database per tests
    let rng = thread_rng();
    let suffix: String = rng.sample_iter(&Alphanumeric).take(16).collect();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL missing from environment.");
    format!("{}_{}", db_url, suffix)
}

fn parse_db_url(db_url: &str) -> (&str, &str) {
    // Create the DB, splitting the url on the last slash
    // postgres://localhost/legasea_test_aoeuaoeu
    let separator_pos = db_url.rfind("/").unwrap();
    let pg_conn = &db_url[..=separator_pos];
    let db_name = &db_url[separator_pos + 1..];
    (pg_conn, db_name)
}

async fn create_db(db_url: &str) {
    let (pg_conn, db_name) = parse_db_url(db_url);

    let mut conn = PgConnection::connect(pg_conn).await.unwrap();

    let sql = format!(r#"CREATE DATABASE "{}""#, &db_name);
    sqlx::query::<Postgres>(&sql)
        .execute(&mut conn)
        .await
        .unwrap();
}

/// For use by TEST code to set up the DB.
async fn drop_db(db_url: &str) {
    let (pg_conn, db_name) = parse_db_url(db_url);
    let mut conn = PgConnection::connect(pg_conn).await.unwrap();

    // Disconnect any existing connections to the DB
    let sql = format!(
        r#"SELECT pg_terminate_backend(pg_stat_activity.pid)
FROM pg_stat_activity
WHERE pg_stat_activity.datname = '{db}'
AND pid <> pg_backend_pid();"#,
        db = db_name
    );
    sqlx::query::<Postgres>(&sql)
        .execute(&mut conn)
        .await
        .unwrap();

    // Clean it up, bubye!
    let sql = format!(r#"DROP DATABASE "{db}";"#, db = db_name);
    sqlx::query::<Postgres>(&sql)
        .execute(&mut conn)
        .await
        .unwrap();
}

async fn run_migrations(db_url: &str) {
    let (pg_conn, db_name) = parse_db_url(db_url);
    let mut conn = PgConnection::connect(&format!("{}/{}", pg_conn, db_name))
        .await
        .unwrap();

    // Run the migrations
    let sql = async_std::fs::read_to_string("bin/setup.sql")
        .await
        .unwrap();

    for query in sql.split(';') {
        sqlx::query::<Postgres>(&query)
            .execute(&mut conn)
            .await
            .unwrap();
    }
}
