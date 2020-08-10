use crate::endpoints::authenticate;
use crate::{BackendApiEndpoint, State};
use async_trait::async_trait;
use chrono::Utc;
use shared::MAX_TWEET_LENGTH;
use shared::{payloads::CreateTweetPayload, responses::TweetResponse, PostTweet};
use sqlx::query;
use tide::{Error, Request, StatusCode};
use uuid::Uuid;

#[async_trait]
impl BackendApiEndpoint for PostTweet {
    async fn handler(
        req: Request<State>,
        create_tweet: CreateTweetPayload,
    ) -> tide::Result<(TweetResponse, StatusCode)> {
        let db_pool = req.state().db_pool.clone();

        if create_tweet.text.len() > MAX_TWEET_LENGTH {
            return Err(Error::from_str(
                StatusCode::UnprocessableEntity,
                format!("Tweet is too long. Max then is {}", MAX_TWEET_LENGTH),
            ));
        }

        let user = authenticate(&req).await?;

        let now = Utc::now();
        let row = query!(
            r#"
            insert into tweets (id, user_id, text, created_at, updated_at)
            values ($1, $2, $3, $4, $5) returning id, text
        "#,
            Uuid::new_v4(),
            user.id,
            create_tweet.text,
            now,
            now,
        )
        .fetch_one(&db_pool)
        .await?;

        Ok((
            TweetResponse {
                id: row.id,
                text: row.text,
            },
            StatusCode::Created,
        ))
    }
}
