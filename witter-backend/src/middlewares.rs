use crate::State;
use tide::Middleware;
use tide::Request;
use tide::Next;
use tide::Response;
use std::future::Future;
use std::pin::Pin;
use serde_json::json;

#[derive(Debug)]
pub struct ErrorReponseToJson;

impl Middleware<State> for ErrorReponseToJson {
    fn handle<'a>(
        &'a self,
        req: Request<State>,
        next: Next<'a, State>,
    ) -> Pin<Box<dyn Future<Output = tide::Result> + Send + 'a>> {
        Box::pin(async move {
            let resp = next.run(req).await;

            match resp {
                Ok(resp) => Ok(resp),
                Err(err) => {
                    let status = err.status();
                    let body = json!({
                        "error": {
                            "message": format!("{}", err),
                        }
                    });

                    Ok(Response::new(status).body_json(&body)?)
                }
            }
        })
    }
}
