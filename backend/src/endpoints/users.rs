use crate::env;
use crate::responses::BuildApiResponse;
use shared::responses::TokenResponse;
use shared::payloads::CreateUserPayload;
use shared::payloads::LoginPayload;
use crate::State;
use argonautica::{Hasher, Verifier};
use chrono::prelude::*;
use failure::Fail;
use futures::compat::Compat01As03;
use rand::distributions::Alphanumeric;
use rand::rngs::OsRng;
use rand::Rng;
use sqlx::query;
use tide::Request;
use tide::StatusCode;
use tide::Response;
use uuid::Uuid;

pub async fn create(mut req: Request<State>) -> tide::Result {
    let db_pool = req.state().db_pool.clone();
    let create_user = req.body_json::<CreateUserPayload>().await?;

    let secret_key = std::env::var("SECRET_KEY")?;
    let clear_text_password = create_user.password.clone();
    let mut hasher = Hasher::default();

    if env::current().is_test() {
        hasher.configure_iterations(1);
    }

    let hashed_password = Compat01As03::new(
        hasher
            .with_password(clear_text_password)
            .with_secret_key(secret_key)
            .hash_non_blocking(),
    )
    .await
    .map_err(|err| err.compat())?;

    let now = Utc::now();
    let row = query!(
        r#"
            insert into users (id, username, hashed_password, created_at, updated_at)
            values ($1, $2, $3, $4, $5) returning id
        "#,
        Uuid::new_v4(),
        create_user.username,
        hashed_password,
        now,
        now,
    )
    .fetch_one(&db_pool)
    .await?;
    let user_id = row.id;

    let raw_token: String = OsRng.sample_iter(&Alphanumeric).take(32).collect();
    let token = query!(
        r#"
                    insert into auth_tokens (
                        id,
                        user_id,
                        token,
                        created_at,
                        updated_at
                    )
                    values ($1, $2, $3, $4, $5) returning token
                "#,
        Uuid::new_v4(),
        user_id,
        raw_token,
        now,
        now,
    )
    .fetch_one(&db_pool)
    .await?;

    TokenResponse::new(&token.token).to_response_with_status(StatusCode::Created)
}

pub async fn login(mut req: Request<State>) -> tide::Result {
    let username = req.param::<String>("username")?;
    let password = req.body_json::<LoginPayload>().await?.password;

    let db_pool = req.state().db_pool.clone();

    let user = query!(
        r#"
            select id, hashed_password
            from users
            where username = $1
        "#,
        username
    )
    .fetch_optional(&db_pool)
    .await?;
    let user = match user {
        Some(user) => user,
        None => return Ok(Response::new(StatusCode::NotFound)),
    };
    let user_password = user.hashed_password;

    let secret_key = std::env::var("SECRET_KEY")?;
    let mut verifier = Verifier::default();
    let is_valid = Compat01As03::new(
        verifier
            .with_hash(user_password)
            .with_password(password)
            .with_secret_key(secret_key)
            .verify_non_blocking(),
    )
    .await
    .map_err(|err| err.compat())?;

    if is_valid {
        let token_row = query!(
            r#"
                        select token
                        from auth_tokens
                        where user_id = $1
                    "#,
            user.id
        )
        .fetch_one(&db_pool)
        .await?;

        TokenResponse::new(&token_row.token).to_response_with_status(StatusCode::Created)
    } else {
        Ok(Response::new(StatusCode::Forbidden))
    }
}
