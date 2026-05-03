use std::marker::PhantomData;

use euclid::{Point2D, Size2D};
use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{
        StoatButton, StoatButtonColorsThemePartialExt, StoatButtonLayoutThemePartialExt,
        Avatar,
    },
    http,
};

#[derive(PartialEq)]
pub struct CurrentUserButton {}

impl Component for CurrentUserButton {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::UserId);

        let current_user_id = radio.slice_current(|state| state.user_id.as_ref().unwrap());
        let current_user = radio.slice(AppChannel::Users, move |state| {
            state.users.get(&*current_user_id.read()).unwrap()
        });
        let mut area = use_state(Area::default);
        let mut show_popup = use_state(|| false);

        rect()
            .horizontal()
            .child(
                Button::new()
                    .flat()
                    .padding(0.)
                    .child(Avatar::new(current_user.clone().into_readable(), None, 42.).presence(true))
                    .corner_radius(42.)
                    .on_press(move |_| show_popup.toggle()),
            )
            .maybe_child(show_popup.read().then(|| {
                rect()
                    .layer(Layer::Overlay)
                    .position(Position::new_absolute().left(56.))
                    .width(Size::px(256.))
                    .corner_radius(4.)
                    .overflow(Overflow::Clip)
                    .shadow(Shadow::new().blur(3.).color(Color::BLACK))
                    .on_sized(move |e: Event<SizedEventData>| area.set(e.area))
                    .on_global_pointer_press(move |e: Event<PointerEventData>| {
                        let area = area.read();
                        let pos = e.global_location();

                        let new_area = euclid::Rect {
                            origin: Point2D {
                                x: area.origin.x as f64,
                                y: area.origin.y as f64,
                                _unit: PhantomData::<()>,
                            },
                            size: Size2D {
                                width: area.size.width as f64,
                                height: area.size.height as f64,
                                _unit: PhantomData::<()>,
                            },
                        };

                        if !new_area.contains(pos) {
                            show_popup.toggle();
                        }
                    })
                    .child(
                        rect()
                            .background(0xff1f1f25)
                            .color(0xffe3e1e9)
                            .padding((8., 0.))
                            .child({
                                let user = current_user.read();

                                StoatButton::new().hover_background(0x14e3e1e9).child(
                                    rect()
                                        .padding((8., 15.))
                                        .color(0xffe3e1e9)
                                        .width(Size::Fill)
                                        .child(
                                            rect()
                                                .horizontal()
                                                .spacing(8.)
                                                .child(
                                                    Avatar::new(current_user.clone().into_readable(), None, 32.)
                                                )
                                                .child(
                                                    rect()
                                                        .child(
                                                            label()
                                                                .text(
                                                                    user.display_name
                                                                        .as_ref()
                                                                        .unwrap_or(&user.username)
                                                                        .clone(),
                                                                )
                                                                .font_size(14),
                                                        )
                                                        .child(
                                                            label()
                                                                .text(format!(
                                                                    "{}#{}",
                                                                    &user.username,
                                                                    &user.discriminator
                                                                ))
                                                                .font_size(12),
                                                        ),
                                                ),
                                        ),
                                )
                            })
                            .child(seperator())
                            .child(presence_button(
                                Color::GREEN,
                                "Online",
                                v0::Presence::Online,
                            ))
                            .child(presence_button(Color::YELLOW, "Idle", v0::Presence::Idle))
                            .child(presence_button(Color::BLUE, "Focus", v0::Presence::Focus))
                            .child(presence_button(
                                Color::RED,
                                "Do Not Disturb",
                                v0::Presence::Busy,
                            ))
                            .child(presence_button(
                                Color::GREY,
                                "Invisible",
                                v0::Presence::Invisible,
                            ))
                            .child(seperator()),
                    )
            }))
            .width(Size::px(42.))
            .height(Size::px(42.))
    }
}

fn seperator() -> Rect {
    rect()
        .margin((4., 0.))
        .width(Size::Fill)
        .height(Size::px(1.))
        .background(0xff45464f)
}

fn presence_button(
    color: impl Into<Color>,
    title: &'static str,
    value: v0::Presence,
) -> StoatButton {
    StoatButton::new()
        .padding((8., 16.))
        .width(Size::Fill)
        .hover_background(0x14e3e1e9)
        .child(
            rect()
                .horizontal()
                .cross_align(Alignment::Center)
                .spacing(8.)
                .on_press(move |_| {
                    let value = value.clone();

                    spawn(async move {
                        http()
                            .edit_user(
                                "@me",
                                &v0::DataEditUser {
                                    display_name: None,
                                    avatar: None,
                                    status: Some(v0::UserStatus {
                                        text: None,
                                        presence: Some(value),
                                    }),
                                    profile: None,
                                    badges: None,
                                    flags: None,
                                    remove: Vec::new(),
                                },
                            )
                            .await
                            .unwrap();
                    });
                })
                .child(
                    rect()
                        .width(Size::px(10.))
                        .height(Size::px(10.))
                        .corner_radius(10.)
                        .overflow(Overflow::Clip)
                        .background(color),
                )
                .child(title)
                .font_size(14),
        )
}
