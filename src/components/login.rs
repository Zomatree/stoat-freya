use std::borrow::Cow;

use freya::prelude::*;

use crate::{
    Config, http,
    types::{DataLogin, MFAResponse, ResponseLogin},
};

#[derive(PartialEq)]
pub struct Login {}

impl Component for Login {
    fn render(&self) -> impl IntoElement {
        let mut config = use_consume::<State<Config>>();

        let email = use_state(String::new);
        let password = use_state(String::new);
        let mut error = use_state(|| None::<String>);

        let mut mfa_ticket = use_state(|| None);
        let mut mfa_value = use_state(String::new);
        let mut mfa_error = use_state(|| None::<String>);

        rect()
            .width(Size::Fill)
            .height(Size::Fill)
            .background(0xff121318)
            .color(0xffe3e1e9)
            .center()
            .child(
                rect()
                    .max_width(Size::px(360.))
                    .max_height(Size::px(600.))
                    .padding((45., 40.))
                    .corner_radius(32.)
                    .background(0xff1f1f25)
                    .spacing(15.)
                    .child(
                        rect()
                            .spacing(8.)
                            .child(label().text("👋 Welcome!").font_size(22))
                            .child(label().text("Sign into Stoat").font_size(16)),
                    )
                    .child(
                        rect()
                            .spacing(15.)
                            .cross_align(Alignment::Center)
                            .child(
                                login_entry(email, "Email", InputMode::Shown).width(Size::px(280.)),
                            )
                            .child(
                                login_entry(password, "Password", InputMode::Hidden('•'))
                                    .width(Size::px(280.)),
                            ),
                    )
                    .child(
                        rect()
                            .width(Size::px(280.))
                            .cross_align(Alignment::Center)
                            .color(0xffb9c3ff)
                            .spacing(32.)
                            .child(
                                rect()
                                    .center()
                                    .child("Reset password")
                                    .height(Size::px(40.)),
                            )
                            .child(
                                rect()
                                    .center()
                                    .child("Resend verification")
                                    .height(Size::px(40.)),
                            ),
                    )
                    .child(
                        rect()
                            .width(Size::px(280.))
                            .horizontal()
                            .spacing(8.)
                            .main_align(Alignment::Center)
                            .child(
                                Button::new()
                                    .child("Exit")
                                    .color(0xffb9c3ff)
                                    .height(Size::px(40.))
                                    .padding((0., 16.))
                                    .corner_radius(40.)
                                    .hover_background(0x20e3e1e9)
                                    .flat()
                                    .on_press(|_| {
                                        let platform = Platform::get();
                                        Platform::get().with_window(None, move |window| {
                                            platform.close_window(window.id());
                                        });
                                    }),
                            )
                            .child(
                                Button::new()
                                    .child("Login")
                                    .height(Size::px(40.))
                                    .padding((0., 16.))
                                    .flat()
                                    .background(0xffb9c3ff)
                                    .color(0xff202c61)
                                    .corner_radius(40.)
                                    .on_press(move |_| {
                                        spawn(async move {
                                            match http()
                                                .login(&DataLogin::Email {
                                                    email: email.read().clone(),
                                                    password: password.read().clone(),
                                                    friendly_name: Some("Stoat-Freya".to_string()),
                                                })
                                                .await
                                            {
                                                Ok(response) => match response {
                                                    ResponseLogin::Success(session) => {
                                                        config.write().token = Some(session.token)
                                                    }
                                                    ResponseLogin::MFA { ticket, .. } => {
                                                        mfa_ticket.set(Some(ticket));
                                                    }
                                                    ResponseLogin::Disabled { .. } => error
                                                        .set(Some("Disabled Account".to_string())),
                                                },
                                                Err(e) => error.set(Some(format!("{e:?}"))),
                                            }
                                        });
                                    }),
                            ),
                    )
                    .maybe_child(error.read().clone()),
            )
            .maybe_child(mfa_ticket.read().cloned().map(|ticket| {
                Popup::new()
                    .background(0xff292a2f)
                    .color(0xffe3e1e9)
                    .show(true)
                    .width(Size::px(370.))
                    .on_close_request(move |_| {
                        mfa_ticket.set(None);
                        mfa_value.set(String::new());
                        mfa_error.set(None);
                    })

                    .child(PopupTitle::new("Confirm action".to_string()))
                    .child(
                        PopupContent::new()
                            .child("Please confirm this action using the selected method.")
                            .child(login_entry(
                                mfa_value,
                                "Authenticator App",
                                InputMode::Shown,
                            ))
                            .maybe_child(mfa_error.read().clone()),
                    )
                    .child(
                        PopupButtons::new()
                            .child(
                                Button::new()
                                    .child("Back")
                                    .color(0xffb9c3ff)
                                    .height(Size::px(40.))
                                    .padding((0., 16.))
                                    .corner_radius(40.)
                                    .hover_background(0x20e3e1e9)
                                    .flat()
                                    .on_press(move |_| {
                                        mfa_ticket.set(None);
                                        mfa_value.set(String::new());
                                        mfa_error.set(None);
                                    }),
                            )
                            .child(
                                Button::new()
                                    .child("Confirm")
                                    .height(Size::px(40.))
                                    .padding((0., 16.))
                                    .flat()
                                    .background(0xffb9c3ff)
                                    .color(0xff202c61)
                                    .corner_radius(40.)
                                    .on_press(move |_| {
                                        let value = mfa_value.read().clone();
                                        let ticket = ticket.clone();

                                        spawn(async move {
                                            match http()
                                                .login(&DataLogin::MFA {
                                                    mfa_ticket: ticket,
                                                    mfa_response: Some(MFAResponse::Totp {
                                                        totp_code: value,
                                                    }),
                                                    friendly_name: Some("Stoat-Freya".to_string()),
                                                })
                                                .await
                                            {
                                                Ok(response) => match response {
                                                    ResponseLogin::Success(session) => {
                                                        mfa_ticket.set(None);
                                                        mfa_value.set(String::new());
                                                        mfa_error.set(None);
                                                        config.write().token = Some(session.token)
                                                    }
                                                    _ => unreachable!(),
                                                },
                                                Err(e) => mfa_error.set(Some(format!("{e:?}"))),
                                            }
                                        });
                                    }),
                            ),
                    )
            }))
    }
}

pub fn login_entry(
    value: impl Into<Writable<String>>,
    placeholder: impl Into<Cow<'static, str>>,
    mode: InputMode,
) -> Rect {
    rect()
        .corner_radius(4.)
        .border(
            Border::new()
                .width(BorderWidth {
                    top: 0.,
                    right: 0.,
                    bottom: 1.,
                    left: 0.,
                })
                .fill(0xffc6c5d0)
                .alignment(BorderAlignment::Inner),
        )
        .background(0xff34343a)
        .color(0xffe3e1e9)
        .padding((10., 8.))
        .center()
        .child(
            Input::new(value)
                .color(0xffe3e1e9)
                .placeholder_color(0xffc6c5d0)
                .placeholder(placeholder)
                .mode(mode)
                .width(Size::Fill)
                .flat()
                .background(Color::TRANSPARENT)
                .hover_background(Color::TRANSPARENT)
                .focus_border_fill(Color::TRANSPARENT),
        )
}
