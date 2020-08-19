use crate::endpoints::authenticate;
use crate::responses::BuildApiResponse;
use crate::State;
use serde::Deserialize;
use shared::{Me, responses::{UserResponse, TweetResponse}, NoPayload};
use sqlx::query_as;
use tide::{StatusCode, Request};
use crate::BackendApiEndpoint;
use async_trait::async_trait;

#[async_trait]
impl BackendApiEndpoint for Me {
    async fn handler(
        req: Request<State>,
        _: NoPayload,
    ) -> tide::Result<(UserResponse, StatusCode)> {
        let user = authenticate(&req).await?;
        Ok((user, StatusCode::Ok))
    }
}

#[derive(Debug, Deserialize)]
struct Pagination {
    page: Option<usize>,
    page_size: Option<usize>,
}

pub async fn timeline(req: Request<State>) -> tide::Result {
    let db_pool = &req.state().db_pool;

    let pagination = req.query::<Pagination>()?;
    let page_size = pagination.page_size.unwrap_or(20).min(20) as i64;

    let page = pagination.page.unwrap_or(1) as i64;
    let offset = (page - 1) * page_size;

    let user = authenticate(&req).await?;

    let tweets = query_as!(
        TweetResponse,
        r#"
            select t.id, t.text
            from (
                select id, text, created_at
                from tweets
                where user_id = $1

                union all

                select tweets.id, tweets.text, tweets.created_at
                from users
                inner join follows on
                    follows.follower_id = $1
                    and follows.followee_id = users.id
                inner join tweets on
                    tweets.user_id = users.id
            ) t
            order by t.created_at desc
            limit $2
            offset $3
        "#,
        user.id,
        page_size,
        offset,
    )
    .fetch_all(db_pool)
    .await?;

    tweets.to_response()
}
