#[allow(unused_imports)]
mod test_helpers;

use test_helpers::*;

#[async_std::test]
async fn creating_a_user() {
    let mut server = test_setup().await;

    let res = get("/users").send(&mut server);
    assert_eq!(res.status(), 200);
    let json = res.body_json::<Value>().await.unwrap();
    assert_json_eq!(json, json!([]));

    // let url = Url::parse("http://example.com/users").unwrap();
    // let mut req = Request::new(Method::Post, url);
    // req.set_body(json!({ "username": "bob" }).to_string());
    // req.set_content_type("application/json".parse().unwrap());
    // let res = server.simulate(req).unwrap();
    // assert_eq!(res.status(), 201);

    // let res = Request::build().get().url("/users").send(&mut server);
    // assert_eq!(res.status(), 200);
    // let json = res.body_json::<Value>().await.unwrap();
    // assert_json_include!(actual: json, expected: json!([{ "username": "bob" }]));
}

// TODO: map invalid data error to some 4xx response
