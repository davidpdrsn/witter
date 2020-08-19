use dotenv;

use async_trait::async_trait;
use payloads::*;
use shared::*;
use sqlx::PgPool;
use sqlx::Pool;
use tide::http::{headers::HeaderValue, Method};
use tide::security::CorsMiddleware;
use tide::security::Origin;
use tide::{Body, Request, Response, Server, StatusCode};

#[cfg(test)]
mod tests;

mod clock;
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

    server.with(
        CorsMiddleware::new()
            .allow_methods(
                "GET, POST, PUT, PATCH, DELETE, OPTIONS"
                    .parse::<HeaderValue>()
                    .unwrap(),
            )
            .allow_origin(Origin::Any),
    );
    server.with(middlewares::ErrorReponseToJson);

    add_endpoint::<CreateUser>(&mut server);

    add_endpoint::<Login>(&mut server);

    server
        .at("/users/:username/session")
        .delete(endpoints::users::logout);

    server
        .at("/users/:username/follow")
        .post(endpoints::users::follow);
    server
        .at("/users/:username/following")
        .get(endpoints::users::following);
    server
        .at("/users/:username/followers")
        .get(endpoints::users::followers);

    add_endpoint::<GetUser>(&mut server);

    add_endpoint::<Me>(&mut server);
    add_endpoint::<Timeline>(&mut server);

    add_endpoint::<PostTweet>(&mut server);

    server
}

#[derive(Debug, Clone)]
pub struct State {
    db_pool: PgPool,
}

#[async_trait]
trait BackendApiEndpoint: ApiEndpoint {
    async fn handler(
        req: Request<State>,
        payload: Self::Payload,
    ) -> tide::Result<(Self::Response, StatusCode)>;
}

#[async_trait]
trait GetRequestPayload: Sized {
    async fn get_payload(req: &mut Request<State>) -> tide::Result<Self>;
}

#[async_trait]
impl GetRequestPayload for NoPayload {
    async fn get_payload(_: &mut Request<State>) -> tide::Result<Self> {
        Ok(NoPayload)
    }
}

macro_rules! impl_get_request_payload {
    ($name:ident) => {
        #[async_trait]
        impl GetRequestPayload for $name {
            async fn get_payload(req: &mut Request<State>) -> tide::Result<Self> {
                req.body_json().await
            }
        }
    };
}

impl_get_request_payload!(CreateTweetPayload);
impl_get_request_payload!(LoginPayload);
impl_get_request_payload!(CreateUserPayload);

fn add_endpoint<E>(server: &mut Server<State>)
where
    E: 'static + BackendApiEndpoint,
    E::Payload: GetRequestPayload + Send,
{
    let mut route = server.at(<E::Url as shared::Url>::URL_SPEC);

    let handler = |mut req: Request<State>| async {
        let payload = E::Payload::get_payload(&mut req).await?;
        let (data, status) = E::handler(req, payload).await?;
        let mut resp = Response::new(status);
        let body = Body::from_json(&serde_json::json!({ "data": data }))?;
        resp.set_body(body);
        Ok(resp)
    };

    match E::METHOD {
        Method::Get => route.get(handler),
        Method::Post => route.post(handler),
        Method::Head => route.head(handler),
        Method::Put => route.put(handler),
        Method::Delete => route.delete(handler),
        Method::Connect => route.connect(handler),
        Method::Options => route.options(handler),
        Method::Trace => route.trace(handler),
        Method::Patch => route.patch(handler),
    };
}
