use http_types::Method;
use serde::{de::DeserializeOwned, Serialize};

pub mod payloads;
pub mod responses;

pub const MAX_TWEET_LENGTH: usize = 140;

pub trait Url {
    const URL_SPEC: &'static str;

    fn url(&self) -> String;
}

pub trait ApiEndpoint {
    type Url: Url;
    const METHOD: Method;
    type Payload;
    type Response: Serialize + DeserializeOwned;
}

pub struct NoPayload;

pub struct GetUser;

impl ApiEndpoint for GetUser {
    type Url = GetUserUrl;
    const METHOD: Method = Method::Get;
    type Payload = NoPayload;
    type Response = responses::UserResponse;
}

pub struct GetUserUrl {
    pub username: String,
}

impl Url for GetUserUrl {
    const URL_SPEC: &'static str = "/users/:username";

    fn url(&self) -> String {
        format!("/users/{}", self.username)
    }
}

pub struct PostTweet;

impl ApiEndpoint for PostTweet {
    type Url = PostTweetUrl;
    const METHOD: Method = Method::Post;
    type Payload = payloads::CreateTweetPayload;
    type Response = responses::TweetResponse;
}

pub struct PostTweetUrl;

impl Url for PostTweetUrl {
    const URL_SPEC: &'static str = "/tweets";

    fn url(&self) -> String {
        format!("/tweets")
    }
}

pub struct Me;

impl ApiEndpoint for Me {
    type Url = MeUrl;
    const METHOD: Method = Method::Get;
    type Payload = NoPayload;
    type Response = responses::UserResponse;
}

pub struct MeUrl;

impl Url for MeUrl {
    const URL_SPEC: &'static str = "/me";

    fn url(&self) -> String {
        format!("/me")
    }
}

pub struct Login;

impl ApiEndpoint for Login {
    type Url = LoginUrl;
    const METHOD: Method = Method::Post;
    type Payload = payloads::LoginPayload;
    type Response = responses::TokenResponse;
}

pub struct LoginUrl {
    pub username: String
}

impl Url for LoginUrl {
    const URL_SPEC: &'static str = "/users/:username/session";

    fn url(&self) -> String {
        format!("/users/{}/session", self.username)
    }
}

pub struct CreateUser;

impl ApiEndpoint for CreateUser {
    type Url = CreateUserUrl;
    const METHOD: Method = Method::Post;
    type Payload = payloads::CreateUserPayload;
    type Response = responses::TokenResponse;
}

pub struct CreateUserUrl;

impl Url for CreateUserUrl {
    const URL_SPEC: &'static str = "/users";

    fn url(&self) -> String {
        format!("/users")
    }
}
