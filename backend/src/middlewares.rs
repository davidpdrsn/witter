use crate::State;
use futures::future::BoxFuture;
use serde_json::json;
use std::future::Future;
use std::pin::Pin;
use tide::http::headers::HeaderValue;
use tide::http::{headers, Method, StatusCode};
use tide::security::Origin;
use tide::Middleware;
use tide::Next;
use tide::Request;
use tide::Response;

#[derive(Debug)]
pub struct ErrorReponseToJson;

#[async_trait::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for ErrorReponseToJson {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        let mut resp = next.run(req).await;

        if let Some(err) = resp.error() {
            let status = err.status();
            let body = json!({
                "error": {
                    "status_code": status.to_string(),
                    "message": format!("{}", err),
                }
            });
            let mut resp = Response::new(status);
            resp.set_body(body);
            Ok(resp)
        } else {
            let status = resp.status();

            if status.is_success() {
                Ok(resp)
            } else {
                let body = resp.take_body();

                if body.is_empty().expect("no length on response body") {
                    let new_body = json!({
                        "error": {
                            "status_code": status.to_string(),
                            "message": "Something went wrong",
                        }
                    });
                    resp.set_body(new_body);
                } else {
                    resp.set_body(body);
                }

                Ok(resp)
            }
        }
    }
}
