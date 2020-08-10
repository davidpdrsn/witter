use crate::{Model, Msg};
use payloads::CreateTweetPayload;
use seed::{prelude::*, *};
use shared::payloads::CreateUserPayload;
use shared::responses::{ApiResponse, TokenResponse, UserResponse};
use shared::Url as _;
use shared::*;

const API_URL: &'static str = "http://localhost:8080";

pub async fn create_user(username: String, password: String) -> Msg {
    let form = CreateUserPayload { username, password };

    let req = Request::new(format!("{}/users", API_URL))
        .method(Method::Post)
        .json(&form)
        .unwrap();
    let resp = seed::browser::fetch::fetch(req).await.unwrap();

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
    let resp = seed::browser::fetch::fetch(req).await.unwrap();

    let user = resp
        .check_status()
        .expect("status check failed")
        .json::<ApiResponse<UserResponse>>()
        .await
        .expect("deserialization failed")
        .data;

    Msg::MeLoaded(user)
}

fn convert_method(method: http_types::Method) -> seed::browser::fetch::Method {
    match method {
        http_types::Method::Get => seed::browser::fetch::Method::Get,
        http_types::Method::Post => seed::browser::fetch::Method::Post,
        http_types::Method::Head => seed::browser::fetch::Method::Head,
        http_types::Method::Put => seed::browser::fetch::Method::Put,
        http_types::Method::Delete => seed::browser::fetch::Method::Delete,
        http_types::Method::Connect => seed::browser::fetch::Method::Connect,
        http_types::Method::Options => seed::browser::fetch::Method::Options,
        http_types::Method::Trace => seed::browser::fetch::Method::Trace,
        http_types::Method::Patch => seed::browser::fetch::Method::Patch,
    }
}

pub trait SetRequestPayload {
    fn set_request_payload<'a>(
        &self,
        req: Request<'a>,
    ) -> seed::browser::fetch::Result<Request<'a>>;
}

impl SetRequestPayload for NoPayload {
    fn set_request_payload<'a>(
        &self,
        req: Request<'a>,
    ) -> seed::browser::fetch::Result<Request<'a>> {
        Ok(req)
    }
}

impl SetRequestPayload for CreateTweetPayload {
    fn set_request_payload<'a>(
        &self,
        req: Request<'a>,
    ) -> seed::browser::fetch::Result<Request<'a>> {
        req.json(self)
    }
}

pub async fn fetch<E>(
    auth_token: Option<String>,
    url: E::Url,
    payload: E::Payload,
    make_msg: fn(E::Response) -> Msg,
) -> Msg
where
    E: ApiEndpoint,
    E::Response: 'static,
    E::Payload: SetRequestPayload,
{
    let result = (|| async {
        let mut req =
            Request::new(format!("{}{}", API_URL, url.url())).method(convert_method(E::METHOD));
        if let Some(auth_token) = auth_token {
            req = req.header(Header::bearer(auth_token));
        }

        req = payload.set_request_payload(req)?;

        let resp = seed::browser::fetch::fetch(req).await?;

        let value = resp
            .check_status()?
            .json::<ApiResponse<E::Response>>()
            .await?
            .data;

        seed::browser::fetch::Result::Ok(make_msg(value))
    })()
    .await;

    match result {
        Ok(msg) => msg,
        Err(err) => Msg::RequestFailed(err),
    }
}
