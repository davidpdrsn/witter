use serde::Serialize;
use tide::http::Error;
use tide::http::StatusCode;
use tide::Response;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }

    pub fn to_response_with_status(self, status: StatusCode) -> Result<Response, Error>
    where
        T: Serialize,
    {
        Ok(Response::new(status).body_json(&self)?)
    }

    #[allow(dead_code)]
    pub fn to_response(self) -> Result<Response, Error>
    where
        T: Serialize,
    {
        Ok(Response::new(StatusCode::Ok).body_json(&self)?)
    }
}

pub trait BuildApiResponse: Serialize + Sized {
    fn to_response_with_status(self, status: StatusCode) -> Result<Response, Error> {
        ApiResponse::new(self).to_response_with_status(status)
    }

    #[allow(dead_code)]
    fn to_response(self) -> Result<Response, Error> {
        ApiResponse::new(self).to_response()
    }
}

impl<T> BuildApiResponse for T where T: Serialize {}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
}

impl TokenResponse {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
}
