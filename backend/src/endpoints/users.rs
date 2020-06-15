use super::authenticate;
use crate::env;
use crate::responses::BuildApiResponse;
use crate::State;
use argonautica::{Hasher, Verifier};
use chrono::prelude::*;
use failure::Fail;
use futures::compat::Compat01As03;
use rand::distributions::Alphanumeric;
use rand::rngs::OsRng;
use rand::Rng;
use serde_json::Value;
use shared::payloads::CreateUserPayload;
use shared::payloads::LoginPayload;
use shared::responses::{TokenResponse, UserResponse};
use sqlx::{query, query_as, PgPool};
use tide::Request;
use tide::Response;
use tide::{Error, StatusCode};
use uuid::Uuid;

pub async fn create(mut req: Request<State>) -> tide::Result {
    let create_user = req.body_json::<CreateUserPayload>().await?;
    let db_pool = &req.state().db_pool;

    if username_already_claimed(&create_user.username, &db_pool).await? {
        return Err(Error::from_str(
            StatusCode::UnprocessableEntity,
            "Username is already claimed",
        ));
    }

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
    .fetch_one(db_pool)
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
    .fetch_one(db_pool)
    .await?;

    TokenResponse::new(&token.token).to_response_with_status(StatusCode::Created)
}

async fn username_already_claimed(username: &str, db_pool: &PgPool) -> tide::Result<bool> {
    let row = query!("select 1 as one from users where username = $1", username)
        .fetch_optional(db_pool)
        .await?;

    Ok(row.is_some())
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

pub async fn follow(req: Request<State>) -> tide::Result {
    let db_pool = req.state().db_pool.clone();
    let current_user = authenticate(&req).await?;
    let username = req.param::<String>("username")?;

    let row = query!("select id from users where username = $1", username)
        .fetch_optional(&db_pool)
        .await?;
    let followee_id: Uuid = if let Some(row) = row { row.id } else { todo!() };

    if current_user.id == followee_id {
        return Err(Error::from_str(
            StatusCode::UnprocessableEntity,
            "You cannot follow yourself",
        ));
    }

    if user_following(current_user.id, followee_id, &db_pool).await? {
        return Err(Error::from_str(
            StatusCode::UnprocessableEntity,
            "You cannot follow the same user twice",
        ));
    }

    let now = Utc::now();
    let rows_inserted = query!(
        r#"
            insert into follows (id, follower_id, followee_id, created_at, updated_at)
            values ($1, $2, $3, $4, $5)
        "#,
        Uuid::new_v4(),
        current_user.id,
        followee_id,
        now,
        now,
    )
    .execute(&db_pool)
    .await?;

    if rows_inserted == 1 {
        Value::Null.to_response_with_status(StatusCode::Created)
    } else {
        todo!()
    }
}

pub async fn following(req: Request<State>) -> tide::Result {
    let db_pool = req.state().db_pool.clone();
    let username = req.param::<String>("username")?;

    // TODO: extract this into function
    let row = query!("select id from users where username = $1", username)
        .fetch_optional(&db_pool)
        .await?;
    let user_id: Uuid = if let Some(row) = row { row.id } else { todo!() };

    let rows = query_as!(
        UserResponse,
        r#"
            select users.id, users.username
            from users
            inner join follows on
                follows.follower_id = $1
                and follows.followee_id = users.id
        "#,
        user_id,
    )
    .fetch_all(&db_pool)
    .await?;

    rows.to_response()
}

pub async fn followers(req: Request<State>) -> tide::Result {
    let db_pool = req.state().db_pool.clone();
    let username = req.param::<String>("username")?;

    let row = query!("select id from users where username = $1", username)
        .fetch_optional(&db_pool)
        .await?;
    let user_id: Uuid = if let Some(row) = row { row.id } else { todo!() };

    let rows = query_as!(
        UserResponse,
        r#"
            select users.id, users.username
            from users
            inner join follows on
                follows.followee_id = $1
                and follows.follower_id = users.id
        "#,
        user_id,
    )
    .fetch_all(&db_pool)
    .await?;

    rows.to_response()
}

async fn user_following(
    current_user_id: Uuid,
    followee_id: Uuid,
    db_pool: &PgPool,
) -> tide::Result<bool> {
    let row = query!(
        r#"
        select 1 as one from follows
        where follower_id = $1 and followee_id = $2
    "#,
        current_user_id,
        followee_id
    )
    .fetch_optional(db_pool)
    .await?;

    Ok(row.is_some())
}
