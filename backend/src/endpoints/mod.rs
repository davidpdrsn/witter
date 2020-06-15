use crate::State;
use shared::responses::UserResponse;
use lazy_static::lazy_static;
use regex::Regex;
use tide::Request;
use tide::http::Error;
use tide::http::StatusCode;
use sqlx::query_as;
use tide::http::headers::HeaderName;

pub mod me;
pub mod users;
pub mod tweets;

lazy_static! {
    static ref BEARER_TOKEN_REGEX: Regex = Regex::new("^Bearer (.*)$").unwrap();
}

pub async fn authenticate(req: &Request<State>) -> Result<UserResponse, Error> {
    let header_value = get_header("Authorization", req)?;

    let caps = match BEARER_TOKEN_REGEX.captures(header_value) {
        Some(caps) => caps,
        None => {
            return Err(Error::from_str(
                StatusCode::BadRequest,
                "Unable to parse Authorization header value",
            ))
        }
    };
    let auth_token = &caps[1];

    let db_pool = &req.state().db_pool;
    let user = query_as!(
        UserResponse,
        r#"
            select users.id, users.username
            from users
            inner join auth_tokens
                on auth_tokens.user_id = users.id
                and auth_tokens.token = $1
            "#,
        auth_token
    )
    .fetch_one(db_pool)
    .await?;

    Ok(user)
}

fn get_header<'a>(header_key: &str, req: &'a Request<State>) -> Result<&'a str, Error> {
    let auth_header_key: HeaderName = header_key.parse()?;

    let header_value = (|| {
        let value = req.header(&auth_header_key)?.get(0)?;
        Some(value.as_str())
    })();

    match header_value {
        Some(value) => Ok(value),
        None => {
            return Err(Error::from_str(
                StatusCode::BadRequest,
                format!("Missing value for `{}` header", header_key),
            ))
        }
    }
}
