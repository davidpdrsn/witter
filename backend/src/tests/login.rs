use crate::tests::test_helpers::*;

#[async_std::test]
async fn authenticating_without_auth_header() {
    let mut server = test_setup().await;

    create_user_and_authenticate(&mut server, None).await;

    let (json, status, headers) = get("/me").send(&mut server).await;
    assert_eq!(status, 400);

    let content_type = &headers["content-type"];
    assert_eq!(content_type, "application/json");

    assert_json_include!(
        actual: json,
        expected: json!({
            "error": {
                "message": "Missing value for `Authorization` header"
            }
        })
    );
}

#[async_std::test]
async fn authenticating_with_invalid_auth_header() {
    let mut server = test_setup().await;

    let token = create_user_and_authenticate(&mut server, None).await.token;

    let (_, status, _) = get("/me")
        .header("Authorization", format!("foo {}", token))
        .send(&mut server)
        .await;
    assert_eq!(status, 400);
}

#[async_std::test]
async fn logging_in_with_unknown_user_gives_404() {
    let mut server = test_setup().await;

    let (_, status, _) = post(
        "/users/bob/session",
        Some(LoginPayload {
            password: "foobar".to_string(),
        }),
    )
    .send(&mut server)
    .await;
    assert_eq!(status, 404);
}

#[async_std::test]
async fn logging_in_with_invalid_password() {
    let mut server = test_setup().await;

    let username = "bob";
    create_user_and_authenticate(&mut server, Some(username.to_string())).await;

    let (json, status, _) = post(
        &format!("/users/{}/session", username),
        Some(LoginPayload {
            password: "baz".to_string(),
        }),
    )
    .send(&mut server)
    .await;
    assert_eq!(status, 403);

    assert_json_include!(
        actual: json,
        expected: json!({
            "error": {
                "status_code": "403",
                "message": "Something went wrong",
            }
        }),
    );
}
