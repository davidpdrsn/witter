use seed::virtual_dom::el_ref::el_ref;
use seed::{prelude::*, *};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::future::Future;
use web_sys::HtmlInputElement;
use seed::browser::fetch::header::Header;

#[derive(Debug)]
struct Model {
    username_input: ElRef<HtmlInputElement>,
    password_input: ElRef<HtmlInputElement>,
    auth_token: Option<String>,
    me: Option<User>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            username_input: ElRef::default(),
            password_input: ElRef::default(),
            auth_token: None,
            me: None,
        }
    }
}

#[derive(Clone)]
enum Msg {
    CreateUserFormSubmitted,
    Authenticated(String),
    MeLoaded(User),
    #[allow(dead_code)]
    Noop,
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Noop => {},
        Msg::Authenticated(token) => {
            model.auth_token = Some(token);

            let auth_token = model.auth_token.clone().unwrap();
            orders.perform_cmd(async move {
                let req = Request::new("http://localhost:8080/me")
                    .header(Header::bearer(auth_token))
                    .method(Method::Get);
                let resp = fetch(req).await.unwrap();

                let user = resp
                    .check_status()
                    .expect("status check failed")
                    .json::<Data<User>>()
                    .await
                    .expect("deserialization failed")
                    .data;

                Msg::MeLoaded(user)
            });
        },
        Msg::MeLoaded(user) => {
            model.me = Some(user);
            log!(model);
        }
        Msg::CreateUserFormSubmitted => {
            let username = model.username_input.get().unwrap().value();
            let password = model.password_input.get().unwrap().value();

            orders.perform_cmd(async {
                let form = CreateUserPayload { username, password };
                let req = Request::new("http://localhost:8080/users")
                    .method(Method::Post)
                    .json(&form)
                    .unwrap();
                let resp = fetch(req).await.unwrap();

                let token = resp
                    .check_status()
                    .expect("status check failed")
                    .json::<Data<Token>>()
                    .await
                    .expect("deserialization failed")
                    .data
                    .token;

                Msg::Authenticated(token)
            });
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateUserPayload {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct Data<T> {
    data: T,
}

#[derive(Debug, Deserialize)]
struct Token {
    token: String,
}

#[derive(Debug, Clone, Deserialize)]
struct User {
    username: String,
    // TODO: id: Uuid,
}

fn view(model: &Model) -> Node<Msg> {
    div![
        div![input![
            el_ref(&model.username_input),
            attrs! { At::Placeholder => "Username" },
        ]],
        div![input![
            el_ref(&model.password_input),
            attrs! { At::Placeholder => "Password" },
        ]],
        div![button![
            "Submit",
            ev(Ev::Click, |_| Msg::CreateUserFormSubmitted),
        ]]
    ]
}

fn after_mount(_: Url, orders: &mut impl Orders<Msg>) -> AfterMount<Model> {
    AfterMount::default()
}

#[wasm_bindgen(start)]
pub fn start() {
    App::builder(update, view)
        .after_mount(after_mount)
        .build_and_start();
}
