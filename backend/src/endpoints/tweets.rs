use crate::endpoints::authenticate;
use crate::responses::BuildApiResponse;
use crate::State;
use chrono::Utc;
use shared::{payloads::CreateTweetPayload, responses::TweetResponse};
use sqlx::query;
use tide::{Error, Request, StatusCode};
use uuid::Uuid;
use shared::MAX_TWEET_LENGTH;

pub async fn create(mut req: Request<State>) -> tide::Result {
    let db_pool = req.state().db_pool.clone();
    let create_tweet = req.body_json::<CreateTweetPayload>().await?;

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

    TweetResponse {
        id: row.id,
        text: row.text,
    }
    .to_response_with_status(StatusCode::Created)
}
