use dotenv;

use argonautica::Hasher;
use argonautica::Verifier;
use async_std::task;
use chrono::prelude::*;
use failure::Fail;
use futures::compat::Compat01As03;
use lazy_static::lazy_static;
use rand::distributions::Alphanumeric;
use rand::rngs::OsRng;
use rand::rngs::ThreadRng;
use rand::RngCore;
use rand::{thread_rng, Rng};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use sqlx::Pool;
use sqlx::{query, query_as};
use tide::http::StatusCode;
use tide::Middleware;
use tide::Request;
use tide::Response;
use tide::Server;
use uuid::Uuid;

#[cfg(test)]
mod tests;

mod env;

#[async_std::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    let db_pool = make_db_pool().await;
    let app = server(db_pool).await;

    app.listen("127.0.0.1:8080").await.unwrap();
}

pub async fn make_db_pool() -> PgPool {
    let db_url = std::env::var("DATABASE_URL").unwrap();
    Pool::new(&db_url).await.unwrap()
}

lazy_static! {
    static ref BEARER_TOKEN_REGEX: Regex = Regex::new("^Bearer (.*)$").unwrap();
}

async fn server(db_pool: PgPool) -> Server<State> {
    let mut server: Server<State> = Server::with_state(State { db_pool });

    server
        .at("/users")
        .get(|req: Request<State>| async move {
            let db_pool = &req.state().db_pool;
            let users = query_as!(User, "select id, username from users")
                .fetch_all(db_pool)
                .await?;

            Ok(Response::new(StatusCode::Ok).body_json(&users)?)
        })
        .post(|mut req: Request<State>| async move {
            let db_pool = req.state().db_pool.clone();
            let create_user = req.body_json::<CreateUser>().await?;

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

            Ok(Response::new(StatusCode::Created).body_json(&json!({
                "data": {
                    "token": token.token,
                }
            }))?)
        });

    server.at("/me").get(|req: Request<State>| async move {
        let auth_header_key = "Authentication".parse()?;
        let header_value = (|| {
            let value = req.header(&auth_header_key)?.get(0)?;
            Some(value.as_str())
        })();
        let header_value = match header_value {
            Some(value) => value,
            None => return Ok(Response::new(StatusCode::BadRequest)),
        };

        let caps = match BEARER_TOKEN_REGEX.captures(header_value) {
            Some(caps) => caps,
            None => return Ok(Response::new(StatusCode::BadRequest)),
        };
        let auth_token = &caps[1];

        let db_pool = &req.state().db_pool;
        let user = query_as!(
            User,
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

        Ok(Response::new(StatusCode::Ok).body_json(&json!({
            "data": {
                "id": user.id,
                "username": user.username
            }
        }))?)
    });

    server
        .at("/users/:username/session")
        .post(|mut req: Request<State>| async move {
            let username = req.param::<String>("username")?;
            let password = req.body_json::<Password>().await?.password;

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

                Ok(Response::new(StatusCode::Created).body_json(&json!({
                    "data": {
                        "token": token_row.token
                    }
                }))?)
            } else {
                Ok(Response::new(StatusCode::Forbidden))
            }
        });

    server
}

#[derive(Debug)]
pub struct State {
    db_pool: PgPool,
}

#[derive(Debug, Serialize)]
struct User {
    id: Uuid,
    username: String,
}

#[derive(Debug, Deserialize)]
struct CreateUser {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct Password {
    password: String,
}
