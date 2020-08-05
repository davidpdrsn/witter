use crate::tests::test_helpers::*;

#[async_std::test]
async fn get_profile_of_other_user() {
    let mut server = test_setup().await;

    let username = "bob";
    create_user_and_authenticate(&mut server, Some(username.to_string())).await;

    let (json, status, _) = get(&format!("/users/{}", username)).send(&mut server).await;
    assert_eq!(status, 200);

    assert_json_include!(
        actual: json,
        expected: json!({
            "data": {
                "username": "bob"
            }
        })
    );
}

#[async_std::test]
async fn get_profile_of_non_unknown_user() {
    let mut server = test_setup().await;

    let (json, status, _) = get("/users/foo").send(&mut server).await;
    assert_eq!(status, 404);

    assert_json_include!(
        actual: json,
        expected: json!({
            "error": {
                "message": "User not found",
                "status_code": "404"
            }
        })
    );
}
