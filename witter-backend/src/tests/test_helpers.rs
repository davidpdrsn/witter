#![allow(dead_code)]

mod test_db;

use serde::Serialize;
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
    TestRequest::Get {
        url: url.to_string(),
    }
}

pub fn post<T: Serialize>(url: &str, body: T) -> TestRequest {
    TestRequest::Post {
        url: url.to_string(),
        body: serde_json::to_value(body).unwrap(),
    }
}

#[derive(Debug)]
pub enum TestRequest {
    Get {
        url: String,
    },
    Post {
        url: String,
        body: Value,
    },
}

impl TestRequest {
    pub fn send(self, server: &mut TestServer<Server<State>>) -> Response {
        let req = match self {
            TestRequest::Get { url } => {
                let url = Url::parse(&format!("http://example.com{}", url)).unwrap();
                Request::new(Method::Get, url)
            }
            TestRequest::Post { url, body } => {
                let url = Url::parse(&format!("http://example.com{}", url)).unwrap();

                let mut req = Request::new(Method::Post, url);
                req.set_body(body.to_string());
                req.set_content_type("application/json".parse().unwrap());
                req
            }
        };

        server.simulate(req).unwrap()
    }
}
