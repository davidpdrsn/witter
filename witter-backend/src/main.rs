use dotenv;

use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use sqlx::Pool;
use sqlx::{query, query_as};
use tide::http::StatusCode;
use tide::Request;
use tide::Response;
use tide::Server;
use uuid::Uuid;

#[cfg(test)]
mod tests;

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

async fn server(db_pool: PgPool) -> Server<State> {
    let mut app: Server<State> = Server::with_state(State { db_pool });

    app.at("/users")
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

            query!(
                r#"
                    insert into users (id, username)
                    values ($1, $2)
                "#,
                Uuid::new_v4(),
                create_user.username,
            )
            .execute(&db_pool)
            .await?;

            Ok(Response::new(StatusCode::Created))
        });

    app
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
}
