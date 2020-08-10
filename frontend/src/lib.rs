use seed::browser::fetch::header::Header;
use seed::virtual_dom::el_ref::el_ref;
use seed::{prelude::*, *};
use shared::payloads::{CreateTweetPayload, CreateUserPayload};
use shared::{
    responses::{ApiResponse, TokenResponse, TweetResponse, UserResponse},
    GetUser, GetUserUrl, NoPayload, PostTweet, PostTweetUrl,
};
use std::fmt;
use web_sys::HtmlInputElement;

mod api;
mod view;

#[derive(Debug)]
pub struct Model {
    login_form: LoginForm,
    sign_up_form: SignUpForm,
    auth_token: Option<String>,
    current_user: Option<UserResponse>,
    page: Page,
}

#[derive(Debug, Default)]
struct LoginForm {
    username_input: ElRef<HtmlInputElement>,
    password_input: ElRef<HtmlInputElement>,
}

#[derive(Debug, Default)]
struct SignUpForm {
    username_input: ElRef<HtmlInputElement>,
    password_input: ElRef<HtmlInputElement>,
}

#[derive(Debug)]
pub enum Page {
    Root,
    Login,
    SignUp,
    UserProfile(String),
}

impl fmt::Display for Page {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Page::Root => write!(f, "/"),
            Page::Login => write!(f, "/login"),
            Page::SignUp => write!(f, "/sign_up"),
            Page::UserProfile(username) => write!(f, "/users/{}", username.clone()),
        }
    }
}

#[derive(Debug)]
pub enum Msg {
    LoginFormSubmitted,
    SignUpFormSubmitted,
    CreateUserEndpointResponded(String),
    MeLoaded(UserResponse),
    UrlChanged(subs::UrlChanged),
    LoadUserProfile(String),
    GetUserLoaded(UserResponse),
    TweetPosted(TweetResponse),
    RequestFailed(FetchError),
    #[allow(dead_code)]
    Noop,
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Noop => {}
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            let page = url_to_page(&url);
            model.page = page;
        }

        Msg::MeLoaded(user) => {
            model.current_user = Some(user);
            log!("me loaded", model);
        }

        Msg::LoginFormSubmitted => {
            // let form = &model.login_form;
            // let username = form.username_input.get().unwrap().value();
            // let password = form.password_input.get().unwrap().value();
            // orders.perform_cmd(api::login(username, password));
        }

        Msg::SignUpFormSubmitted => {
            let form = &model.sign_up_form;
            let username = form.username_input.get().unwrap().value();
            let password = form.password_input.get().unwrap().value();
            orders.perform_cmd(api::create_user(username, password));
        }
        Msg::CreateUserEndpointResponded(token) => {
            model.auth_token = Some(token.clone());
            orders.perform_cmd(api::reload_current_user(token.to_string()));
        }

        Msg::LoadUserProfile(username) => {
            orders.perform_cmd(api::fetch::<GetUser>(
                model.auth_token.clone(),
                GetUserUrl { username },
                NoPayload,
                Msg::GetUserLoaded,
            ));

            orders.perform_cmd(api::fetch::<PostTweet>(
                model.auth_token.clone(),
                PostTweetUrl,
                CreateTweetPayload {
                    text: "Tweet text".to_string(),
                },
                Msg::TweetPosted,
            ));
        }
        Msg::GetUserLoaded(user) => log!("user loaded", user),
        Msg::TweetPosted(tweet) => log!(tweet),
        Msg::RequestFailed(err) => log!("request failed", err),
    }
}

fn url_to_page(url: &Url) -> Page {
    let path = url.path().iter().map(|s| s.as_str()).collect::<Vec<_>>();

    match path.as_slice() {
        ["sign_up"] => Page::SignUp,
        ["login"] => Page::Login,
        ["users", username] => Page::UserProfile(username.to_string()),
        [] => Page::Root,
        _ => todo!(),
    }
}

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.subscribe(Msg::UrlChanged);
    orders.send_msg(Msg::UrlChanged(subs::UrlChanged(url.clone())));

    let page = url_to_page(&url);

    match &page {
        Page::UserProfile(username) => {
            orders.send_msg(Msg::LoadUserProfile(username.to_string()));
        }
        _ => {}
    }

    Model {
        auth_token: None,
        current_user: None,
        page,
        login_form: Default::default(),
        sign_up_form: Default::default(),
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view::view);
}
