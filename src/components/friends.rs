use freya::{
    icons::lucide::{bell, chevron_left, circle_slash, hand, inbox, plus, users_round},
    prelude::*,
};

use crate::components::FriendsList;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub enum FriendPage {
    #[default]
    Online,
    All,
    Pending,
    Blocked,
}

#[derive(PartialEq)]
pub struct Friends {}

impl Component for Friends {
    fn render(&self) -> impl IntoElement {
        let mut page = use_state(FriendPage::default);

        rect()
            .child(
                rect()
                    .horizontal()
                    .height(Size::px(48.))
                    .padding((0., 16.))
                    .spacing(10.)
                    .margin((8., 8., 8., 0.))
                    .cross_align(Alignment::Center)
                    .child(
                        rect()
                            .cross_align(Alignment::Center)
                            .horizontal()
                            .child(
                                svg(chevron_left())
                                    .width(Size::px(20.))
                                    .height(Size::px(20.)),
                            )
                            .child(
                                svg(users_round())
                                    .width(Size::px(24.))
                                    .height(Size::px(24.)),
                            ),
                    )
                    .child(label().text("Friends").font_size(16)),
            )
            .child(
                rect().horizontal().child(
                    rect()
                        .margin((0., 8., 8., 8.))
                        .width(Size::Fill)
                        .height(Size::Fill)
                        .corner_radius(28.)
                        .background(0xff0d0e13)
                        .overflow(Overflow::Clip)
                        .child(
                            rect()
                                .padding((0., 8.))
                                .horizontal()
                                .child(
                                    rect()
                                        // .padding((14., 0.))
                                        .width(Size::px(56.))
                                        .cross_align(Alignment::Center)
                                        .child(
                                            rect()
                                                .margin((12., 0.))
                                                .background(0xffb9c3ff)
                                                .corner_radius(12.)
                                                .width(Size::px(40.))
                                                .height(Size::px(40.))
                                                .center()
                                                .child(
                                                    svg(plus())
                                                        .width(Size::px(24.))
                                                        .height(Size::px(24.))
                                                        .color(0xff202c61),
                                                ),
                                        )
                                        .children(
                                            [
                                                (hand(), "Online", FriendPage::Online),
                                                (inbox(), "All", FriendPage::All),
                                                (bell(), "Pending", FriendPage::Pending),
                                                (circle_slash(), "Blocked", FriendPage::Blocked),
                                            ]
                                            .into_iter()
                                            .map(
                                                |(icon, title, value)| {
                                                    rect()
                                                        .width(Size::px(56.))
                                                        .cross_align(Alignment::Center)
                                                        .on_press(move |_| {
                                                            page.set_if_modified(value)
                                                        })
                                                        .child(
                                                            rect()
                                                                .width(Size::px(56.))
                                                                .height(Size::px(32.))
                                                                .corner_radius(56.)
                                                                .maybe(
                                                                    *page.read() == value,
                                                                    |this| {
                                                                        this.background(0xff424659)
                                                                    },
                                                                )
                                                                .center()
                                                                .child(
                                                                    svg(icon)
                                                                        .width(Size::px(24.))
                                                                        .height(Size::px(24.)),
                                                                ),
                                                        )
                                                        .child(
                                                            label()
                                                                .text(title)
                                                                .font_size(12)
                                                                .margin((4., 0.)),
                                                        )
                                                        .into_element()
                                                },
                                            ),
                                        ), // .child(Button::new().child("Online").on_press(move |_| {
                                           //     page.set_if_modified(FriendPage::Online)
                                           // }))
                                           // .child(Button::new().child("All").on_press(move |_| {
                                           //     page.set_if_modified(FriendPage::All)
                                           // }))
                                           // .child(Button::new().child("Pending").on_press(move |_| {
                                           //     page.set_if_modified(FriendPage::Pending)
                                           // }))
                                           // .child(Button::new().child("Blocked").on_press(
                                           //     move |_| page.set_if_modified(FriendPage::Blocked),
                                           // )),
                                )
                                .child(FriendsList { page }),
                        ),
                ),
            )
    }
}
