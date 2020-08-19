use crate::{Model, Msg, Page, flash::FlashMsg};
use seed::{prelude::*, *};

pub fn view(model: &Model) -> Node<Msg> {
    div![
        flash(model),
        nav(model),
        view_page(model),
    ]
}

fn view_page(model: &Model) -> Node<Msg> {
    match &model.page {
        Page::Root => p!["Welcome!"],
        Page::Login => login(model),
        Page::SignUp => sign_up(model),
        Page::UserProfile(username) => user_profile(username),
        Page::SignedIn => signed_in(),
    }
}

fn flash(model: &Model) -> Node<Msg> {
    match model.flash.get() {
        None => div![],
        Some(FlashMsg::Notice(msg)) => div![
            "NOTICE: ", msg
        ],
        Some(FlashMsg::Error(msg)) => div![
            "ERROR: ", msg
        ],
    }
}

fn signed_in() -> Node<Msg> {
    div!["Signed in!"]
}

fn nav(model: &Model) -> Node<Msg> {
    if let Some(current_user) = &model.current_user {
        div![
            a!["Home", attrs! { At::Href => Page::Root }],
            " | ",
            a![
                &current_user.username,
                attrs! { At::Href => Page::UserProfile(current_user.username.clone()) }
            ],
            " | ",
            a!["Logout", ev(Ev::Click, |_| Msg::Logout), attrs! { At::Href => "#" }],
        ]
    } else {
        div![
            a!["Home", attrs! { At::Href => Page::Root }],
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
