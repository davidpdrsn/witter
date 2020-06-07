use crate::State;
use futures::future::BoxFuture;
use tide::http::{headers, Method, StatusCode};
use serde_json::json;
use std::future::Future;
use std::pin::Pin;
use tide::http::headers::HeaderValue;
use tide::security::Origin;
use tide::Middleware;
use tide::Next;
use tide::Request;
use tide::Response;

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

pub const WILDCARD: &str = "*";
pub const DEFAULT_MAX_AGE: &str = "86400";
pub const DEFAULT_METHODS: &str = "GET, POST, OPTIONS";

#[derive(Clone, Debug, Hash)]
pub struct CorsMiddleware {
    allow_credentials: Option<HeaderValue>,
    allow_headers: HeaderValue,
    allow_methods: HeaderValue,
    allow_origin: Origin,
    expose_headers: Option<HeaderValue>,
    max_age: HeaderValue,
}

impl CorsMiddleware {
    /// Creates a new Cors middleware.
    pub fn new() -> Self {
        Self {
            allow_credentials: None,
            allow_headers: WILDCARD.parse().unwrap(),
            allow_methods: DEFAULT_METHODS.parse().unwrap(),
            allow_origin: Origin::Any,
            expose_headers: None,
            max_age: DEFAULT_MAX_AGE.parse().unwrap(),
        }
    }

    /// Set allow_credentials and return new Cors
    pub fn allow_credentials(mut self, allow_credentials: bool) -> Self {
        self.allow_credentials = match allow_credentials.to_string().parse() {
            Ok(header) => Some(header),
            Err(_) => None,
        };
        self
    }

    /// Set allow_headers and return new Cors
    pub fn allow_headers<T: Into<HeaderValue>>(mut self, headers: T) -> Self {
        self.allow_headers = headers.into();
        self
    }

    /// Set max_age and return new Cors
    pub fn max_age<T: Into<HeaderValue>>(mut self, max_age: T) -> Self {
        self.max_age = max_age.into();
        self
    }

    /// Set allow_methods and return new Cors
    pub fn allow_methods<T: Into<HeaderValue>>(mut self, methods: T) -> Self {
        self.allow_methods = methods.into();
        self
    }

    /// Set allow_origin and return new Cors
    pub fn allow_origin<T: Into<Origin>>(mut self, origin: T) -> Self {
        self.allow_origin = origin.into();
        self
    }

    /// Set expose_headers and return new Cors
    pub fn expose_headers<T: Into<HeaderValue>>(mut self, headers: T) -> Self {
        self.expose_headers = Some(headers.into());
        self
    }

    /// Determine if origin is appropriate
    fn is_valid_origin(&self, origin: &HeaderValue) -> bool {
        let origin = origin.as_str().to_string();

        match &self.allow_origin {
            Origin::Any => true,
            Origin::Exact(s) => s == &origin,
            Origin::List(list) => list.contains(&origin),
        }
    }

    fn build_preflight_response(&self, origin: &[HeaderValue]) -> tide::http::Response {
        let mut response = tide::http::Response::new(StatusCode::Ok);
        response
            .insert_header(headers::ACCESS_CONTROL_ALLOW_ORIGIN, origin.clone())
            .unwrap();
        response
            .insert_header(
                headers::ACCESS_CONTROL_ALLOW_METHODS,
                self.allow_methods.clone(),
            )
            .unwrap();
        response
            .insert_header(
                headers::ACCESS_CONTROL_ALLOW_HEADERS,
                self.allow_headers.clone(),
            )
            .unwrap();
        response
            .insert_header(headers::ACCESS_CONTROL_MAX_AGE, self.max_age.clone())
            .unwrap();

        if let Some(allow_credentials) = self.allow_credentials.clone() {
            response
                .insert_header(headers::ACCESS_CONTROL_ALLOW_CREDENTIALS, allow_credentials)
                .unwrap();
        }

        if let Some(expose_headers) = self.expose_headers.clone() {
            response
                .insert_header(headers::ACCESS_CONTROL_EXPOSE_HEADERS, expose_headers)
                .unwrap();
        }

        response
    }

    /// Look at origin of request and determine allow_origin
    fn response_origin(&self, origin: &HeaderValue) -> Option<HeaderValue> {
        if !self.is_valid_origin(origin) {
            return None;
        }

        match self.allow_origin {
            Origin::Any => Some(WILDCARD.parse().unwrap()),
            _ => Some(origin.clone()),
        }
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for CorsMiddleware {
    fn handle<'a>(
        &'a self,
        req: Request<State>,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, tide::Result> {
        Box::pin(async move {
            // TODO: how should multiple origin values be handled?
            let origins = req.header(&headers::ORIGIN).cloned();

            if origins.is_none() {
                // This is not a CORS request if there is no Origin header
                return next.run(req).await;
            }

            let origins = origins.unwrap();
            let origin = origins.last();

            if !self.is_valid_origin(origin) {
                return Ok(tide::http::Response::new(StatusCode::Unauthorized).into());
            }

            // Return results immediately upon preflight request
            if req.method() == Method::Options {
                return Ok(self.build_preflight_response(&[origin.clone()]).into());
            }

            let mut response: tide::http::Response = match next.run(req).await {
                Ok(resp) => resp.into(),
                Err(err) => Response::new(err.status())
                    .body_string(format!("{}", err))
                    .into(),
            };

            response
                .insert_header(
                    headers::ACCESS_CONTROL_ALLOW_ORIGIN,
                    self.response_origin(&origin).unwrap(),
                )
                .unwrap();

            if let Some(allow_credentials) = self.allow_credentials.clone() {
                response
                    .insert_header(headers::ACCESS_CONTROL_ALLOW_CREDENTIALS, allow_credentials)
                    .unwrap();
            }

            if let Some(expose_headers) = self.expose_headers.clone() {
                response
                    .insert_header(headers::ACCESS_CONTROL_EXPOSE_HEADERS, expose_headers)
                    .unwrap();
            }
            Ok(response.into())
        })
    }
}
