use seed::{prelude::*, *};
use shared::responses::{TweetResponse, UserResponse};
use std::fmt;
use web_sys::HtmlInputElement;
use flash::Flash;

mod api;
mod storage;
mod view;
mod flash;

#[derive(Debug)]
pub struct Model {
    login_form: LoginForm,
    sign_up_form: SignUpForm,
    auth_token: Option<String>,
    current_user: Option<UserResponse>,
    page: Page,
    flash: Flash,
}

impl Model {
    fn set_auth_token(&mut self, token: &str) {
        self.auth_token = Some(token.to_string());
        storage::set_auth_token(token);
    }

    fn remove_auth_token(&mut self) {
        self.auth_token = None;
        self.current_user = None;
        storage::remove_auth_token();
    }
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
    SignedIn,
}

impl Page {
    fn go(self, model: &mut Model, orders: &mut impl Orders<Msg>) {
        let url = self.to_string().parse::<Url>().expect("not a URL");
        seed::browser::service::routing::push_route(url);

        self.load_data(orders);

        model.page = self;
    }

    fn load_data(&self, orders: &mut impl Orders<Msg>) {
        match self {
            Page::UserProfile(username) => {
                orders.send_msg(Msg::LoadUserProfile(username.to_string()));
            }
            Page::Root | Page::Login | Page::SignUp | Page::SignedIn => {}
        }
    }
}

impl From<Url> for Page {
    fn from(url: Url) -> Self {
        let path = url.path().iter().map(|s| s.as_str()).collect::<Vec<_>>();

        match path.as_slice() {
            ["sign_up"] => Page::SignUp,
            ["login"] => Page::Login,
            ["users", username] => Page::UserProfile(username.to_string()),
            [] => Page::Root,
            ["signed_in"] => Page::SignedIn,
            _ => todo!("Unknown URL: {}", url),
        }
    }
}

impl fmt::Display for Page {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Page::Root => write!(f, "/"),
            Page::Login => write!(f, "/login"),
            Page::SignUp => write!(f, "/sign_up"),
            Page::UserProfile(username) => write!(f, "/users/{}", username.clone()),
            Page::SignedIn => write!(f, "/signed_in"),
        }
    }
}

#[derive(Debug)]
pub enum Msg {
    LoginFormSubmitted,
    SignUpFormSubmitted,
    LoginEndpointResponded(String),
    CreateUserEndpointResponded(String),
    MeLoaded(UserResponse),
    UrlChanged(subs::UrlChanged),
    LoadUserProfile(String),
    GetUserLoaded(UserResponse),
    TweetPosted(TweetResponse),
    Error(Error),
    Logout,
    ClearFlash,
    #[allow(dead_code)]
    Noop,
}

#[derive(Debug)]
pub enum Error {
    RequestFailed(FetchError),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Noop => {}
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            let page = Page::from(url);
            model.page = page;
        }

        Msg::MeLoaded(user) => {
            model.current_user = Some(user);
        }

        Msg::LoginFormSubmitted => {
            let form = &model.login_form;
            let username = form.username_input.get().unwrap().value();
            let password = form.password_input.get().unwrap().value();
            orders.perform_cmd(api::login(username, password));
        }
        Msg::LoginEndpointResponded(token) => {
            model.set_auth_token(&token);
            orders.perform_cmd(api::reload_current_user(token.to_string()));
            Page::SignedIn.go(model, orders);
        }

        Msg::SignUpFormSubmitted => {
            let form = &model.sign_up_form;
            let username = form.username_input.get().unwrap().value();
            let password = form.password_input.get().unwrap().value();
            orders.perform_cmd(api::create_user(username, password));
        }
        Msg::CreateUserEndpointResponded(token) => {
            model.set_auth_token(&token);
            orders.perform_cmd(api::reload_current_user(token.to_string()));
            Page::SignedIn.go(model, orders);
        }

        Msg::LoadUserProfile(username) => {
            orders.perform_cmd(api::load_user(username, model.auth_token.clone()));
        }
        Msg::GetUserLoaded(user) => log!("user loaded", user),
        Msg::TweetPosted(tweet) => log!(tweet),
        Msg::Error(err) => match err {
            Error::RequestFailed(err) => {
                log!("request failed", err);
                model.flash.set_error("Request failed", orders);
            }
        }

        Msg::ClearFlash => {
            model.flash.clear();
        }

        Msg::Logout => {
            Page::Root.go(model, orders);
            model.remove_auth_token();
        }
    }
}

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.subscribe(Msg::UrlChanged);
    orders.send_msg(Msg::UrlChanged(subs::UrlChanged(url.clone())));

    let page = Page::from(url);

    page.load_data(orders);

    let model = Model {
        auth_token: storage::get_auth_token(),
        current_user: None,
        page,
        login_form: Default::default(),
        sign_up_form: Default::default(),
        flash: Default::default(),
    };

    if let Some(token) = &model.auth_token {
        orders.perform_cmd(api::reload_current_user(token.clone()));
    }

    model
}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view::view);
}
