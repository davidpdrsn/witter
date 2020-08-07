use crate::Msg;
use seed::{prelude::*, *};
use shared::payloads::CreateUserPayload;
use shared::responses::{ApiResponse, TokenResponse, UserResponse};

const API_URL: &'static str = "http://localhost:8080";

pub async fn create_user(username: String, password: String) -> Msg {
    let form = CreateUserPayload {
        username,
        password,
    };

    let req = Request::new(format!("{}/users", API_URL))
        .method(Method::Post)
        .json(&form)
        .unwrap();
    let resp = fetch(req).await.unwrap();

    let token = resp
        .check_status()
        .expect("status check failed")
        .json::<ApiResponse<TokenResponse>>()
        .await
        .expect("deserialization failed")
        .data
        .token;

    Msg::CreateUserEndpointResponded(token)
}

pub async fn reload_current_user(auth_token: String) -> Msg {
    let req = Request::new(format!("{}/me", API_URL))
        .header(Header::bearer(auth_token))
        .method(Method::Get);
    let resp = fetch(req).await.unwrap();

    let user = resp
        .check_status()
        .expect("status check failed")
        .json::<ApiResponse<UserResponse>>()
        .await
        .expect("deserialization failed")
        .data;

    Msg::MeLoaded(user)
}
