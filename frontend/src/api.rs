use crate::{Model, Msg, Error};
use payloads::{CreateTweetPayload, LoginPayload};
use seed::{prelude::*, *};
use shared::payloads::CreateUserPayload;
use shared::responses::{ApiResponse, TokenResponse, UserResponse};
use shared::Url as _;
use shared::*;

const API_URL: &'static str = "http://localhost:8080";

pub async fn create_user(username: String, password: String) -> Msg {
    fetch::<CreateUser>(
        None,
        CreateUserUrl,
        CreateUserPayload { username, password },
        |resp| Msg::CreateUserEndpointResponded(resp.token),
    )
    .await
}

pub async fn login(username: String, password: String) -> Msg {
    fetch::<Login>(
        None,
        LoginUrl { username },
        LoginPayload { password },
        |resp| Msg::LoginEndpointResponded(resp.token),
    )
    .await
}

pub async fn reload_current_user(auth_token: String) -> Msg {
    fetch::<Me>(Some(auth_token), MeUrl, NoPayload, Msg::MeLoaded).await
}

pub async fn load_user(username: String, auth_token: Option<String>) -> Msg {
    fetch::<GetUser>(
        auth_token,
        GetUserUrl { username: username.to_string() },
        NoPayload,
        Msg::GetUserLoaded,
    ).await
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
        Err(err) => Msg::Error(Error::RequestFailed(err)),
    }
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

macro_rules! impl_set_request_payload {
    ($name:ident) => {
        impl SetRequestPayload for $name {
            fn set_request_payload<'a>(
                &self,
                req: Request<'a>,
            ) -> seed::browser::fetch::Result<Request<'a>> {
                req.json(self)
            }
        }
    };
}

impl_set_request_payload!(CreateTweetPayload);
impl_set_request_payload!(LoginPayload);
impl_set_request_payload!(CreateUserPayload);
