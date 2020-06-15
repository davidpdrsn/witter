use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TweetResponse {
    pub id: Uuid,
    pub text: String,
}
