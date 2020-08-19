use crate::endpoints::authenticate;
use crate::responses::BuildApiResponse;
use crate::BackendApiEndpoint;
use crate::State;
use async_trait::async_trait;
use serde::Deserialize;
use shared::{
    responses::{TweetResponse, UserResponse},
    ApiEndpoint, Me, NoPayload, Timeline,
};
use sqlx::{query, query_as};
use tide::{Request, StatusCode};

#[async_trait]
impl BackendApiEndpoint for Me {
    async fn handler(
        req: Request<State>,
        _: NoPayload,
    ) -> tide::Result<(<Self as ApiEndpoint>::Response, StatusCode)> {
        let user = authenticate(&req).await?;
        Ok((user, StatusCode::Ok))
    }
}

#[derive(Debug, Deserialize)]
struct Pagination {
    page: Option<usize>,
    page_size: Option<usize>,
}

#[async_trait]
impl BackendApiEndpoint for Timeline {
    async fn handler(
        req: Request<State>,
        _: NoPayload,
    ) -> tide::Result<(<Self as ApiEndpoint>::Response, StatusCode)> {
        let db_pool = &req.state().db_pool;

        let pagination = req.query::<Pagination>()?;
        let page_size = pagination.page_size.unwrap_or(20).min(20) as i64;

        let page = pagination.page.unwrap_or(1) as i64;
        let offset = (page - 1) * page_size;

        let current_user = authenticate(&req).await?;

        let tweets = query!(
            r#"
            select
                tweets.id as tweet_id
                , tweets.text as tweet_text
                , tweets.created_at as tweet_created_at
                , users.id as user_id
                , users.username as user_username
            from (
                select id, text, created_at, user_id
                from tweets
                where user_id = $1

                union all

                select tweets.id, tweets.text, tweets.created_at, tweets.user_id
                from users
                inner join follows on
                    follows.follower_id = $1
                    and follows.followee_id = users.id
                inner join tweets on
                    tweets.user_id = users.id
            ) tweets
            inner join users on users.id = tweets.user_id
            order by tweets.created_at desc
            limit $2
            offset $3
        "#,
            current_user.id,
            page_size,
            offset,
        )
        .fetch_all(db_pool)
        .await?;

        let tweet_responses = tweets
            .into_iter()
            .map(|tweet| TweetResponse {
                id: tweet.tweet_id.unwrap(),
                text: tweet.tweet_text.unwrap(),
                created_at: tweet.tweet_created_at.unwrap(),
                user: UserResponse {
                    id: tweet.user_id,
                    username: tweet.user_username,
                },
            })
            .collect::<Vec<_>>();

        Ok((tweet_responses, StatusCode::Ok))
    }
}
