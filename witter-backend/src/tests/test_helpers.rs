#![allow(dead_code)]

use crate::Server;
use crate::State;
use crate::{make_db_pool, server};
use futures::{executor::block_on, prelude::*};
use http_service::{HttpService, Response};
use serde::de::DeserializeOwned;
use sqlx::prelude::Connect;
use sqlx::PgConnection;
use sqlx::PgPool;
use sqlx::Postgres;
use std::env;
use std::pin::Pin;

pub use assert_json_diff::{assert_json_eq, assert_json_include};
pub use http_types::Request;
pub use http_types::{Method, Url};
pub use serde_json::{json, Value};

pub async fn test_server() -> TestBackend<Server<State>> {
    dotenv::dotenv().ok();

    let db_pool = make_db_pool().await;
    let server = server(db_pool).await;
    make_server(server).unwrap()
}

#[derive(Debug)]
pub struct TestBackend<T: HttpService> {
    service: T,
    connection: T::Connection,
}

impl<T: HttpService> TestBackend<T> {
    fn wrap(service: T) -> Result<Self, <T::ConnectionFuture as TryFuture>::Error> {
        let connection = block_on(service.connect().into_future())?;
        Ok(Self {
            service,
            connection,
        })
    }

    pub fn simulate(
        &mut self,
        req: Request,
    ) -> Result<Response, <T::ResponseFuture as TryFuture>::Error> {
        block_on(
            self.service
                .respond(self.connection.clone(), req)
                .into_future(),
        )
    }
}

pub fn make_server<T: HttpService>(
    service: T,
) -> Result<TestBackend<T>, <T::ConnectionFuture as TryFuture>::Error> {
    TestBackend::wrap(service)
}

pub trait BodyJson {
    fn body_json<T: DeserializeOwned>(
        self,
    ) -> Pin<Box<dyn Future<Output = Result<T, Box<dyn std::error::Error>>>>>;
}

impl BodyJson for Response {
    fn body_json<T: DeserializeOwned>(
        self,
    ) -> Pin<Box<dyn Future<Output = Result<T, Box<dyn std::error::Error>>>>> {
        Box::pin(async move {
            let body = self.body_string().await?;
            Ok(serde_json::from_str(&body)?)
        })
    }
}

pub trait MakeRequestBuilder {
    fn build() -> RequestBuilder;
}

impl MakeRequestBuilder for Request {
    fn build() -> RequestBuilder {
        RequestBuilder::default()
    }
}

#[derive(Debug, Default)]
pub struct RequestBuilder {
    method: Option<Method>,
    url: Option<String>,
}

impl RequestBuilder {
    pub fn get(mut self) -> Self {
        self.method = Some(Method::Get);
        self
    }

    pub fn post(mut self) -> Self {
        self.method = Some(Method::Post);
        self
    }

    pub fn patch(mut self) -> Self {
        self.method = Some(Method::Patch);
        self
    }

    pub fn put(mut self) -> Self {
        self.method = Some(Method::Put);
        self
    }

    pub fn delete(mut self) -> Self {
        self.method = Some(Method::Delete);
        self
    }

    pub fn url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }

    pub fn send(self, server: &mut TestBackend<Server<State>>) -> Response {
        let url = Url::parse(&format!("http://example.com{}", self.url.unwrap())).unwrap();
        let req = Request::new(Method::Get, url);
        server.simulate(req).unwrap()
    }
}

pub fn db_url() -> String {
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
    let (pg_conn, db_name) = dbg!(parse_db_url(db_url));

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

pub async fn run_migrations(db_url: &str) {
    let (pg_conn, db_name) = parse_db_url(db_url);
    let mut conn = PgConnection::connect(&format!("{}/{}", pg_conn, db_name))
        .await
        .unwrap();

    let rows = sqlx::query!(
        "SELECT table_name FROM information_schema.tables WHERE table_schema='public'"
    )
    .fetch_all(&mut conn)
    .await
    .unwrap();
    dbg!(rows);

    // Run the migrations
    let sql = async_std::fs::read_to_string("bin/setup.sql")
        .await
        .unwrap();
    sqlx::query::<Postgres>(&sql)
        .execute(&mut conn)
        .await
        .unwrap();
}

pub struct TestDb {
    db_url: String,
    db_pool: Option<PgPool>,
}

/// Sets up a new DB for running tests with.
impl TestDb {
    pub async fn new() -> Self {
        let db_url = db_url();
        dbg!(&db_url);
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
