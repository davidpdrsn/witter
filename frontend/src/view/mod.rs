use crate::{flash::FlashMsg, Model, Msg, Page, PageData};
use seed::{prelude::*, *};
use shared::responses::TweetResponse;

pub fn view(model: &Model) -> Vec<Node<Msg>> {
    nodes![flash(model), nav(model), view_page(model),]
}

fn view_page(model: &Model) -> Node<Msg> {
    match &model.page {
        Page::RootLoggedOut => p!["Welcome"],
        Page::Login => login(model),
        Page::SignUp => sign_up(model),
        Page::UserProfile(username) => user_profile(username),
        Page::SignedIn => signed_in(),
        Page::PostTweet => post_tweet(model),
        Page::Timeline(tweets) => timeline(model, tweets),
    }
}

fn timeline(model: &Model, tweets: &PageData<Vec<TweetResponse>>) -> Node<Msg> {
    match tweets {
        PageData::NotLoaded => p!["Loading..."],
        PageData::Loaded(tweets) => {
            let tweets_views: Vec<Node<Msg>> = tweets.iter().map(tweet).collect::<Vec<_>>();
            div![tweets_views]
        }
    }
}

fn tweet(tweet: &TweetResponse) -> Node<Msg> {
    div![
        a![
            "@",
            &tweet.user.username,
            attrs! {
                At::Href => Page::UserProfile(tweet.user.username.to_string())
            }
        ],
        br![],
        &tweet.text,
        br![],
        format!("{:?}", &tweet.created_at),
        hr![],
    ]
}

fn post_tweet(model: &Model) -> Node<Msg> {
    div![
        div![input![
            el_ref(&model.post_tweet_form.text_input),
            attrs! {
                At::Type => "text",
                At::Placeholder => "Whats up?",
            },
        ]],
        div![button![
            "Post",
            ev(Ev::Click, |_| Msg::PostTweetFormSubmitted)
        ]]
    ]
}

fn flash(model: &Model) -> Node<Msg> {
    match model.flash.get() {
        None => div![],
        Some(FlashMsg::Notice(msg)) => div!["NOTICE: ", msg],
        Some(FlashMsg::Error(msg)) => div!["ERROR: ", msg],
    }
}

fn signed_in() -> Node<Msg> {
    div!["Signed in!"]
}

fn nav(model: &Model) -> Node<Msg> {
    if let Some(current_user) = &model.current_user {
        div![
            a![
                "Home",
                attrs! { At::Href => Page::Timeline(PageData::NotLoaded) }
            ],
            " | ",
            a!["Post tweet", attrs! { At::Href => Page::PostTweet }],
            " | ",
            a![
                &current_user.username,
                attrs! { At::Href => Page::UserProfile(current_user.username.clone()) }
            ],
            " | ",
            a![
                "Logout",
                ev(Ev::Click, |_| Msg::Logout),
                attrs! { At::Href => "#" }
            ],
        ]
    } else {
        div![
            a!["Home", attrs! { At::Href => Page::RootLoggedOut }],
            " | ",
            a!["Login", attrs! { At::Href => Page::Login }],
            " | ",
            a!["Sign up", attrs! { At::Href => Page::SignUp }],
        ]
    }
}

fn login(model: &Model) -> Node<Msg> {
    div![
        div![input![
            el_ref(&model.login_form.username_input),
            attrs! {
                At::Type => "text",
                At::Placeholder => "Username",
            },
        ]],
        div![input![
            el_ref(&model.login_form.password_input),
            attrs! {
                At::Type => "password",
                At::Placeholder => "Password"
            },
        ]],
        div![button!["Login", ev(Ev::Click, |_| Msg::LoginFormSubmitted),]]
    ]
}

fn sign_up(model: &Model) -> Node<Msg> {
    div![
        div![input![
            el_ref(&model.sign_up_form.username_input),
            attrs! {
                At::Type => "text",
                At::Placeholder => "Username",
            },
        ]],
        div![input![
            el_ref(&model.sign_up_form.password_input),
            attrs! {
                At::Type => "password",
                At::Placeholder => "Password"
            },
        ]],
        div![button![
            "Sign up",
            ev(Ev::Click, |_| Msg::SignUpFormSubmitted),
        ]]
    ]
}

fn user_profile(username: &str) -> Node<Msg> {
    p!["Profile of ", username]
}
