use std::borrow::Cow;

use freya::{
    icons::lucide::{moon, x},
    prelude::*,
};

use crate::{
    Config, components::{StoatButton, StoatButtonColorsThemePartialExt, StoatButtonLayoutThemePartialExt}, http, theme::Theme, types::{DataLogin, MFAResponse, ResponseLogin}, use_material_theme
};

#[derive(PartialEq)]
pub struct Login {}

impl Component for Login {
    fn render(&self) -> impl IntoElement {
        let mut config = use_consume::<State<Config>>();
        let theme = use_material_theme();

        let email = use_state(String::new);
        let password = use_state(String::new);
        let mut error = use_state(|| None::<String>);

        let mut mfa_ticket = use_state(|| None::<String>);
        let mut mfa_value = use_state(String::new);
        let mut mfa_error = use_state(|| None::<String>);

        rect()
            .width(Size::Fill)
            .height(Size::Fill)
            .background(theme.md.surface.as_argb_u32())
            .center()
            .child(
                rect()
                    .width(Size::Fill)
                    .height(Size::Fill)
                    .padding((40., 35.))
                    .cross_align(Alignment::Center)
                    .main_align(Alignment::SpaceBetween)
                    .child(
                        rect()
                            .width(Size::Fill)
                            .horizontal()
                            .main_align(Alignment::End)
                            .child(
                                StoatButton::new()
                                    .background(theme.md.secondary_container.as_argb_u32())
                                    .color(theme.md.on_secondary_container.as_argb_u32())
                                    .corner_radius(40.)
                                    .on_press(move |_| config.write().theme.scheme.toggle())
                                    .child(
                                        rect()
                                            .width(Size::px(40.))
                                            .height(Size::px(40.))
                                            .center()
                                            .child(
                                                svg(moon())
                                                    .width(Size::px(24.))
                                                    .height(Size::px(24.)),
                                            ),
                                    ),
                            ),
                    )
                    .child(
                        rect()
                            .max_width(Size::px(360.))
                            .max_height(Size::px(600.))
                            .padding((45., 40.))
                            .corner_radius(32.)
                            .background(theme.md.surface_container.as_argb_u32())
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
                                        login_entry(email, "Email", &theme, InputMode::Shown)
                                            .width(Size::px(280.)),
                                    )
                                    .child(
                                        login_entry(
                                            password,
                                            "Password",
                                            &theme,
                                            InputMode::Hidden('•'),
                                        )
                                        .width(Size::px(280.)),
                                    ),
                            )
                            .child(
                                rect()
                                    .width(Size::px(280.))
                                    .cross_align(Alignment::Center)
                                    .color(theme.md.primary.as_argb_u32())
                                    .spacing(32.)
                                    .child(
                                        StoatButton::new()
                                            .color(theme.md.primary.as_argb_u32())
                                            .corner_radius(40.)
                                            .child(
                                                rect()
                                                    .height(Size::px(40.))
                                                    .padding((0., 16.))
                                                    .center()
                                                    .child("Reset password"),
                                            ),
                                    )
                                    .child(
                                        StoatButton::new()
                                            .color(theme.md.primary.as_argb_u32())
                                            .corner_radius(40.)
                                            .child(
                                                rect()
                                                    .height(Size::px(40.))
                                                    .padding((0., 16.))
                                                    .center()
                                                    .child("Resend verification"),
                                            ),
                                    ),
                            )
                            .child(
                                rect()
                                    .width(Size::px(280.))
                                    .horizontal()
                                    .spacing(8.)
                                    .main_align(Alignment::Center)
                                    .child(
                                        StoatButton::new()
                                            .color(theme.md.primary.as_argb_u32())
                                            .corner_radius(40.)
                                            .child(
                                                rect()
                                                    .padding((0., 16.))
                                                    .height(Size::px(40.))
                                                    .horizontal()
                                                    .center()
                                                    .spacing(4.)
                                                    .child(
                                                        svg(x())
                                                            .width(Size::px(12.))
                                                            .height(Size::px(12.)),
                                                    )
                                                    .child("Exit"),
                                            )
                                            .on_press(|_| {
                                                let platform = Platform::get();
                                                Platform::get().with_window(None, move |window| {
                                                    platform.close_window(window.id());
                                                });
                                            }),
                                    )
                                    .child(
                                        StoatButton::new()
                                            .child(
                                                rect()
                                                    .height(Size::px(40.))
                                                    .padding((0., 16.))
                                                    .center()
                                                    .child("Login"),
                                            )
                                            .background(theme.md.primary.as_argb_u32())
                                            .color(theme.md.on_primary.as_argb_u32())
                                            .corner_radius(40.)
                                            .on_press(move |_| {
                                                spawn(async move {
                                                    match http()
                                                        .login(&DataLogin::Email {
                                                            email: email.read().clone(),
                                                            password: password.read().clone(),
                                                            friendly_name: Some(
                                                                "Stoat-Freya".to_string(),
                                                            ),
                                                        })
                                                        .await
                                                    {
                                                        Ok(response) => match response {
                                                            ResponseLogin::Success(session) => {
                                                                config.write().token =
                                                                    Some(session.token)
                                                            }
                                                            ResponseLogin::MFA {
                                                                ticket, ..
                                                            } => {
                                                                mfa_ticket.set(Some(ticket));
                                                            }
                                                            ResponseLogin::Disabled { .. } => error
                                                                .set(Some(
                                                                    "Disabled Account".to_string(),
                                                                )),
                                                        },
                                                        Err(e) => error.set(Some(format!("{e:?}"))),
                                                    }
                                                });
                                            }),
                                    ),
                            )
                            .maybe_child(error.read().clone()),
                    )
                    .child(rect().height(Size::px(32.)).width(Size::Fill).color(theme.md.on_surface_variant.as_argb_u32()).child("Developed by Zomatree"))
            )
            .maybe_child(mfa_ticket.read().cloned().map(|ticket| {
                Popup::new()
                    .background(0xff292a2f)
                    .color(0xffe3e1e9)
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
                                &theme,
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
    theme: &Theme,
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
                .fill(theme.md.on_surface_variant.as_argb_u32())
                .alignment(BorderAlignment::Inner),
        )
        .background(theme.md.surface_container_highest.as_argb_u32())
        .padding((10., 8.))
        .center()
        .child(
            Input::new(value)
                .color(theme.md.on_surface.as_argb_u32())
                .placeholder_color(theme.md.on_surface_variant.as_argb_u32())
                .placeholder(placeholder)
                .mode(mode)
                .width(Size::Fill)
                .flat()
                .background(Color::TRANSPARENT)
                .focus_background(Color::TRANSPARENT)
                .focus_border_fill(Color::TRANSPARENT),
        )
}
