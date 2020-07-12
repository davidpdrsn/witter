use crate::tests::test_helpers::*;

#[async_std::test]
async fn creating_a_user_and_logging_in() {
    let mut server = test_setup().await;

    let token = create_user_and_authenticate(&mut server, Some("bob".to_string()))
        .await
        .token;

    let (json, status, _) = get("/me")
        .header("Authorization", format!("Bearer {}", token))
        .send(&mut server)
        .await;
    assert_eq!(status, 200);

    assert_json_include!(
        actual: json,
        expected: json!({
            "data": {
                "username": "bob"
            }
        })
    );

    let (json, status, _) = post(
        "/users/bob/session",
        Some(LoginPayload {
            password: "foobar".to_string(),
        }),
    )
    .send(&mut server)
    .await;
    assert_eq!(status, 201);
    assert_json_include!(
        actual: json,
        expected: json!({
            "data": {
                "token": token
            }
        })
    );
}

#[async_std::test]
async fn claiming_username_already_claimed_gives_client_error() {
    let mut server = test_setup().await;

    let username = "bob".to_string();

    create_user_and_authenticate(&mut server, Some(username.clone())).await;

    let (json, status, _) = post(
        "/users",
        Some(CreateUserPayload {
            username,
            password: "bar".to_string(),
        }),
    )
    .send(&mut server)
    .await;

    assert_eq!(status, 422);
    assert_json_include!(
        actual: json,
        expected: json!({
            "error": {
                "message": "Username is already claimed"
            }
        })
    );
}
