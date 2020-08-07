use crate::tests::test_helpers::*;

#[async_std::test]
async fn logging_out() {
    let mut server = test_setup().await;

    let token = create_user_and_authenticate(&mut server, Some("bob".to_string()))
        .await
        .token;

    let (_, status, _) = get("/me")
        .header("Authorization", format!("Bearer {}", token))
        .send(&mut server)
        .await;
    assert_eq!(status, 200);

    let (_, status, _) = delete("/users/bob/session")
        .header("Authorization", format!("Bearer {}", token))
        .send(&mut server)
        .await;
    assert_eq!(status, 200);

    let (_, status, _) = get("/me")
        .header("Authorization", format!("Bearer {}", token))
        .send(&mut server)
        .await;
    assert_eq!(status, 403);
}
