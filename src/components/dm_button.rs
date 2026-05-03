use std::borrow::Cow;

use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{HomeSelection, Avatar, image},
};

#[derive(PartialEq)]
pub struct DMButton {
    pub channel: Readable<v0::Channel>,
    pub selection: State<HomeSelection>,
}

impl Component for DMButton {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::UserId);
        let user_id = radio.read().user_id.clone().unwrap();

        rect()
            .padding((0., 8., 0., 8.))
            .spacing(8.)
            .height(Size::px(42.))
            .width(Size::Fill)
            .main_align(Alignment::Center)
            .corner_radius(42.)
            .overflow(Overflow::Clip)
            .color(0xff90909a)
            .maybe(
                self.selection.read().channel_id() == Some(self.channel.read().id()),
                |btn| btn.background(0xff384379).color(0xffdde1ff),
            )
            .child(match self.channel.read().clone() {
                v0::Channel::DirectMessage { recipients, .. } => {
                    let other = recipients
                        .iter()
                        .find(|&id| id != &user_id)
                        .unwrap()
                        .clone();

                    let user = radio.slice(AppChannel::Users, move |state| {
                        state.users.get(&other).unwrap()
                    });

                    DMDirectMessageButton {
                        user: user.into_readable(),
                    }
                    .into_element()
                }
                v0::Channel::Group { .. } => DMGroupButton {
                    channel: self.channel.clone(),
                }
                .into_element(),
                _ => unreachable!(),
            })
            .on_press({
                let channel = self.channel.clone();
                let mut selection = self.selection.clone();

                move |_| {
                    let id = channel.read().id().to_string();

                    *selection.write() = HomeSelection::DM(id);
                }
            })
            .on_pointer_enter(|_| {
                Cursor::set(CursorIcon::Pointer);
            })
            .on_pointer_leave(|_| {
                Cursor::set(CursorIcon::Default);
            })
    }
}

#[derive(PartialEq)]
pub struct DMDirectMessageButton {
    pub user: Readable<v0::User>,
}

impl Component for DMDirectMessageButton {
    fn render(&self) -> impl IntoElement {
        let user = use_memo({
            let user = self.user.clone();
            move || user.read().clone()
        });

        rect()
            .horizontal()
            .height(Size::Fill)
            .cross_align(Alignment::Center)
            .spacing(8.)
            .child(
                Avatar::new(self.user.clone(), None, 32.).presence(true)
            )
            .child(
                rect()
                    .child(
                        label()
                            .text({
                                let user = user.read();
                                user.display_name.as_ref().unwrap_or(&user.username).clone()
                            })
                            .font_size(15)
                            .max_lines(1)
                            .text_overflow(TextOverflow::Ellipsis),
                    )
                    .maybe_child(
                        user.read()
                            .status
                            .as_ref()
                            .and_then(|status| {
                                status
                                    .text
                                    .as_ref()
                                    .map(|text| Cow::Owned(text.clone()))
                                    .or(status.presence.as_ref().map(|presence| match presence {
                                        v0::Presence::Online => Cow::Borrowed("Online"),
                                        v0::Presence::Idle => Cow::Borrowed("Idle"),
                                        v0::Presence::Focus => Cow::Borrowed("Focus"),
                                        v0::Presence::Busy => Cow::Borrowed("Busy"),
                                        v0::Presence::Invisible => Cow::Borrowed("Invisible"),
                                    }))
                            })
                            .map(|text| {
                                label()
                                    .text(text)
                                    .font_size(11)
                                    .max_lines(1)
                                    .text_overflow(TextOverflow::Ellipsis)
                            }),
                    ),
            )
    }
}

#[derive(PartialEq)]
pub struct DMGroupButton {
    pub channel: Readable<v0::Channel>,
}

impl Component for DMGroupButton {
    fn render(&self) -> impl IntoElement {
        let (name, icon, users) = match &*self.channel.read() {
            v0::Channel::Group {
                name,
                icon,
                recipients,
                ..
            } => (name.clone(), icon.clone(), recipients.len()),
            _ => unreachable!(),
        };

        rect()
            .horizontal()
            .spacing(8.)
            .child(
                rect()
                    .width(Size::px(32.))
                    .height(Size::px(32.))
                    .corner_radius(12.)
                    .overflow(Overflow::Clip)
                    .child(match icon {
                        Some(icon) => image(&icon).into_element(),
                        None => {
                            let initials = name
                                .trim()
                                .split_whitespace()
                                .filter_map(|run| run.chars().next())
                                .take(2)
                                .collect::<String>();

                            rect()
                                .background(0xffb9c3ff)
                                .width(Size::Fill)
                                .height(Size::Fill)
                                .center()
                                .font_size(12)
                                .child(initials)
                                .color(0xff202c61)
                                .into_element()
                        }
                    }),
            )
            .child(
                rect()
                    .child(label().text(name).font_size(15))
                    .child(label().text(format!("{users} Members")).font_size(11)),
            )
    }
}
