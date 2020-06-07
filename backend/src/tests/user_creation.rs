use crate::tests::test_helpers::*;

#[async_std::test]
async fn creating_a_user_and_logging_in() {
    let mut server = test_setup().await;

    let res = post(
        "/users",
        CreateUserPayload {
            username: "bob".to_string(),
            password: "foobar".to_string(),
        },
    )
    .send(&mut server)
    .await;
    assert_eq!(res.status(), 201);

    let resp = res.body_json::<ApiResponse<TokenResponse>>().await.unwrap();
    let token = resp.data.token;

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
        LoginPayload {
            password: "foobar".to_string(),
        },
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

    let res = post(
        "/users",
        CreateUserPayload {
            username: username.clone(),
            password: "foo".to_string(),
        },
    )
    .send(&mut server)
    .await;
    assert_eq!(res.status(), 201);

    let res = post(
        "/users",
        CreateUserPayload {
            username,
            password: "bar".to_string(),
        },
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
