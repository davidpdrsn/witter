use crate::tests::test_helpers::*;

#[async_std::test]
async fn creating_a_user_and_logging_in() {
    let mut server = test_setup().await;

    let token = create_user_and_authenticate(&mut server, Some("bob".to_string()))
        .await
        .token;

    let res = get("/me")
        .header("Authorization", format!("Bearer {}", token))
        .send(&mut server)
        .await;
    assert_eq!(res.status(), 200);

    let json = res.body_json::<Value>().await.unwrap();
    assert_json_include!(
        actual: json,
        expected: json!({
            "data": {
                "username": "bob"
            }
        })
    );

    let res = post(
        "/users/bob/session",
        Some(LoginPayload {
            password: "foobar".to_string(),
        }),
    )
    .send(&mut server)
    .await;
    assert_eq!(res.status(), 201);
    let json = res.body_json::<Value>().await.unwrap();
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

    let res = post(
        "/users",
        Some(CreateUserPayload {
            username,
            password: "bar".to_string(),
        }),
    )
    .send(&mut server)
    .await;
    assert_eq!(res.status(), 422);

    let json = res.body_json::<Value>().await.unwrap();

    assert_json_include!(
        actual: json,
        expected: json!({
            "error": {
                "message": "Username is already claimed"
            }
        })
    );
}
