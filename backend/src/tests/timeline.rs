use crate::tests::test_helpers::*;

#[async_std::test]
async fn sees_own_tweets() {
    let mut server = test_setup().await;

    let token = create_user_and_authenticate(&mut server, None).await.token;

    post_tweet("oldest", &token, &mut server).await;
    post_tweet("middle", &token, &mut server).await;
    post_tweet("newest", &token, &mut server).await;

    let (json, status, _) = get("/me/timeline")
        .header("Authorization", format!("Bearer {}", token))
        .send(&mut server)
        .await;

    assert_eq!(status, 200);
    assert_json_include!(
        actual: json,
        expected: json!({
            "data": [
                { "text": "newest" },
                { "text": "middle" },
                { "text": "oldest" },
            ]
        })
    );
}

#[async_std::test]
async fn sees_tweets_from_users_we_are_following() {
    let mut server = test_setup().await;

    let bob_token = create_user_and_authenticate(&mut server, Some("bob".to_string()))
        .await
        .token;
    let alice_token = create_user_and_authenticate(&mut server, Some("alice".to_string()))
        .await
        .token;

    post_tweet("oldest", &alice_token, &mut server).await;
    post_tweet("middle", &alice_token, &mut server).await;
    post_tweet("newest", &alice_token, &mut server).await;

    let (_, status, _) = empty_post("/users/alice/follow")
        .header("Authorization", format!("Bearer {}", bob_token))
        .send(&mut server)
        .await;
    assert_eq!(status, 201);

    let (json, status, _) = get("/me/timeline")
        .header("Authorization", format!("Bearer {}", bob_token))
        .send(&mut server)
        .await;

    assert_eq!(status, 200);
    assert_json_include!(
        actual: json,
        expected: json!({
            "data": [
                { "text": "newest" },
                { "text": "middle" },
                { "text": "oldest" },
            ]
        })
    );
}

#[async_std::test]
async fn pagination() {
    let mut server = test_setup().await;

    let token = create_user_and_authenticate(&mut server, None).await.token;

    post_tweet("5", &token, &mut server).await;
    post_tweet("4", &token, &mut server).await;
    post_tweet("3", &token, &mut server).await;
    post_tweet("2", &token, &mut server).await;
    post_tweet("1", &token, &mut server).await;

    // page 1
    let (json, status, _) = get("/me/timeline?page=1&page_size=2")
        .header("Authorization", format!("Bearer {}", token))
        .send(&mut server)
        .await;

    assert_eq!(status, 200);
    assert_json_include!(
        actual: &json,
        expected: json!({
            "data": [
                { "text": "1" },
                { "text": "2" },
            ]
        })
    );
    assert_eq!(json["data"].as_array().unwrap().len(), 2);

    // page 2
    let (json, status, _) = get("/me/timeline?page=2&page_size=2")
        .header("Authorization", format!("Bearer {}", token))
        .send(&mut server)
        .await;

    assert_eq!(status, 200);
    assert_json_include!(
        actual: &json,
        expected: json!({
            "data": [
                { "text": "3" },
                { "text": "4" },
            ]
        })
    );
    assert_eq!(json["data"].as_array().unwrap().len(), 2);

    // page 3
    let (json, status, _) = get("/me/timeline?page=3&page_size=2")
        .header("Authorization", format!("Bearer {}", token))
        .send(&mut server)
        .await;

    assert_eq!(status, 200);
    assert_json_include!(
        actual: &json,
        expected: json!({
            "data": [
                { "text": "5" },
            ]
        })
    );
    assert_eq!(json["data"].as_array().unwrap().len(), 1);
}

#[async_std::test]
async fn max_page_size() {
    let mut server = test_setup().await;

    let token = create_user_and_authenticate(&mut server, None).await.token;

    for _ in 0..21 {
        post_tweet("hi", &token, &mut server).await;
    }

    let (json, status, _) = get("/me/timeline?page=1&page_size=100")
        .header("Authorization", format!("Bearer {}", token))
        .send(&mut server)
        .await;

    assert_eq!(status, 200);
    assert_eq!(json["data"].as_array().unwrap().len(), 20);
}

async fn post_tweet(text: &str, token: &str, server: &mut TestServer) {
    post(
        "/tweets",
        Some(CreateTweetPayload {
            text: text.to_string(),
        }),
    )
    .header("Authorization", format!("Bearer {}", token))
    .send(server)
    .await;
}
