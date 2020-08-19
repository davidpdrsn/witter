use flash::Flash;
use seed::{prelude::*, *};
use shared::responses::{PostTweetResponse, TweetResponse, UserResponse};
use std::fmt;
use web_sys::HtmlInputElement;

mod api;
mod flash;
mod storage;
mod view;

#[derive(Debug)]
pub struct Model {
    login_form: LoginForm,
    sign_up_form: SignUpForm,
    post_tweet_form: PostTweetForm,
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

    fn logged_in(&self) -> bool {
        self.auth_token.is_some()
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

#[derive(Debug, Default)]
struct PostTweetForm {
    text_input: ElRef<HtmlInputElement>,
}

#[derive(Debug)]
pub enum PageData<T> {
    Loaded(T),
    NotLoaded,
}

#[derive(Debug)]
pub enum Page {
    RootLoggedOut,
    Timeline(PageData<Vec<TweetResponse>>),
    Login,
    SignUp,
    UserProfile(String),
    SignedIn,
    PostTweet,
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
            Page::Timeline(_) => {
                orders.send_msg(Msg::LoadTimeline);
            }
            Page::RootLoggedOut | Page::Login | Page::SignUp | Page::SignedIn | Page::PostTweet => {
            }
        }
    }

    fn from(url: Url, model: &Model) -> Self {
        let path = url.path().iter().map(|s| s.as_str()).collect::<Vec<_>>();

        match path.as_slice() {
            ["sign_up"] => Page::SignUp,
            ["login"] => Page::Login,
            ["users", username] => Page::UserProfile(username.to_string()),
            [] => {
                if model.logged_in() {
                    Page::Timeline(PageData::NotLoaded)
                } else {
                    Page::RootLoggedOut
                }
            }
            ["signed_in"] => Page::SignedIn,
            ["tweets", "new"] => Page::PostTweet,
            _ => todo!("Unknown URL: {}", url),
        }
    }
}

impl fmt::Display for Page {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Page::RootLoggedOut => write!(f, "/"),
            Page::Timeline(_) => write!(f, "/"),
            Page::Login => write!(f, "/login"),
            Page::SignUp => write!(f, "/sign_up"),
            Page::UserProfile(username) => write!(f, "/users/{}", username.clone()),
            Page::SignedIn => write!(f, "/signed_in"),
            Page::PostTweet => write!(f, "/tweets/new"),
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
    LoadTimelineEndpointResponded(Vec<TweetResponse>),
    LoadTimeline,
    PostTweetFormSubmitted,
    PostTweetEndpointResponded(PostTweetResponse),
    #[allow(dead_code)]
    Noop,
}

#[derive(Debug)]
pub enum Error {
    RequestFailed(FetchError),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    log!("received message", msg);

    match msg {
        Msg::Noop => {}
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            let page = Page::from(url, model);
            page.load_data(orders);
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
        },

        Msg::ClearFlash => {
            model.flash.clear();
        }

        Msg::Logout => {
            Page::RootLoggedOut.go(model, orders);
            model.remove_auth_token();
        }

        Msg::LoadTimelineEndpointResponded(tweets) => {
            if let Page::Timeline(data) = &mut model.page {
                *data = PageData::Loaded(tweets);
            }
        }
        Msg::LoadTimeline => {
            orders.perform_cmd(api::load_timeline(model.auth_token.clone()));
        }

        Msg::PostTweetFormSubmitted => {
            let text = model.post_tweet_form.text_input.get().unwrap().value();
            orders.perform_cmd(api::post_tweet(model.auth_token.clone(), text));
        }
        Msg::PostTweetEndpointResponded(_) => {
            model.flash.set_notice("Tweet posted", orders);
            Page::Timeline(PageData::NotLoaded).go(model, orders);
        }
    }
}

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.subscribe(Msg::UrlChanged);
    orders.send_msg(Msg::UrlChanged(subs::UrlChanged(url.clone())));

    let mut model = Model {
        auth_token: storage::get_auth_token(),
        current_user: None,
        page: Page::RootLoggedOut,
        login_form: Default::default(),
        sign_up_form: Default::default(),
        post_tweet_form: Default::default(),
        flash: Default::default(),
    };

    let page = Page::from(url, &model);
    page.load_data(orders);
    model.page = page;

    if let Some(token) = &model.auth_token {
        orders.perform_cmd(api::reload_current_user(token.clone()));
    }

    model
}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view::view);
}
