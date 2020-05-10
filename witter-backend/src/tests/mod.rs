#[allow(unused_imports)]
mod test_helpers;

// TODO: map invalid data error to some 4xx response

use test_helpers::*;

#[async_std::test]
async fn creating_a_user() {
    let mut server = test_setup().await;

    let res = get("/users").send(&mut server);
    assert_eq!(res.status(), 200);
    let json = res.body_json::<Value>().await.unwrap();
    assert_json_eq!(json, json!([]));

    let res = post("/users", json!({ "username": "bob" })).send(&mut server);
    assert_eq!(res.status(), 201);

    let res = get("/users").send(&mut server);
    assert_eq!(res.status(), 200);
    let json = res.body_json::<Value>().await.unwrap();
    assert_json_include!(actual: json, expected: json!([{ "username": "bob" }]));
}
