use dotenv;

use sqlx::PgPool;
use sqlx::Pool;
use tide::http::headers::HeaderValue;
use tide::security::Origin;
use tide::Server;

#[cfg(test)]
mod tests;

mod endpoints;
mod env;
mod middlewares;
mod responses;

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
    let mut server: Server<State> = Server::with_state(State { db_pool });

    server.middleware(
        middlewares::CorsMiddleware::new()
            .allow_methods(
                "GET, POST, PUT, PATCH, DELETE, OPTIONS"
                    .parse::<HeaderValue>()
                    .unwrap(),
            )
            .allow_origin(Origin::Any),
    );

    server.middleware(middlewares::ErrorReponseToJson);

    server.at("/users").post(endpoints::users::create);
    server
        .at("/users/:username/session")
        .post(endpoints::users::login);
    server
        .at("/users/:username/follow")
        .post(endpoints::users::follow);
    server
        .at("/users/:username/following")
        .get(endpoints::users::following);
    server
        .at("/users/:username/followers")
        .get(endpoints::users::followers);

    server.at("/me").get(endpoints::me::get);
    server.at("/me/timeline").get(endpoints::me::timeline);

    server.at("/tweets").post(endpoints::tweets::create);

    server
}

#[derive(Debug)]
pub struct State {
    db_pool: PgPool,
}
