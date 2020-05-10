#![allow(dead_code)]

mod test_db;

use crate::Server;
use crate::State;
use crate::{make_db_pool, server};
use futures::{executor::block_on, prelude::*};
use http_service::{HttpService, Response};
use serde::de::DeserializeOwned;
use sqlx::PgConnection;
use sqlx::PgPool;
use sqlx::Postgres;
use sqlx::prelude::Connect;
use std::env;
use std::pin::Pin;
use test_db::TestDb;

pub use assert_json_diff::{assert_json_eq, assert_json_include};
pub use http_types::Request;
pub use http_types::{Method, Url};
pub use serde_json::{json, Value};

pub async fn test_setup() -> TestServer<Server<State>> {
    let test_db = TestDb::new().await;
    let db_pool = test_db.db();

    let server = server(db_pool).await;
    TestServer::new(server, test_db).unwrap()
}

#[derive(Debug)]
pub struct TestServer<T: HttpService> {
    service: T,
    connection: T::Connection,
    test_db: TestDb,
}

impl<T: HttpService> TestServer<T> {
    fn new(service: T, test_db: TestDb) -> Result<Self, <T::ConnectionFuture as TryFuture>::Error> {
        let connection = block_on(service.connect().into_future())?;
        Ok(Self {
            service,
            connection,
            test_db,
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

pub fn get(url: &str) -> TestRequest {
    TestRequest {
        url: url.to_string(),
    }
}

#[derive(Debug)]
pub struct TestRequest {
    url: String,
}

impl TestRequest {
    pub fn send(self, server: &mut TestServer<Server<State>>) -> Response {
        let url = Url::parse(&format!("http://example.com{}", self.url)).unwrap();
        let req = Request::new(Method::Get, url);
        server.simulate(req).unwrap()
    }
}
