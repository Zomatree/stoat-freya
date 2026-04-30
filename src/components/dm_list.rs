use freya::{
    icons::lucide::{house, notebook_text, users_round},
    prelude::*,
    radio::use_radio,
};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{DMButton, HomeSelection},
    http,
};

#[derive(PartialEq)]
pub struct DMList {
    pub selection: State<HomeSelection>,
}

impl Component for DMList {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Channels);

        let saved_messages = use_memo({
            let radio = radio.clone();

            move || {
                radio
                    .read()
                    .channels
                    .values()
                    .find(|channel| matches!(channel, v0::Channel::SavedMessages { .. }))
                    .cloned()
            }
        });

        let channels = use_memo(move || {
            let mut channels = radio
                .read()
                .channels
                .values()
                .filter_map(|channel| match channel {
                    v0::Channel::DirectMessage { id, .. } | v0::Channel::Group { id, .. } => {
                        let id = id.clone();

                        Some(
                            radio
                                .slice_current(move |state| state.channels.get(&id).unwrap())
                                .into_readable(),
                        )
                    }
                    _ => None,
                })
                .collect::<Vec<Readable<v0::Channel>>>();

            channels.sort_by(|a, b| {
                let (id_a, last_msg_a) = match &*a.read() {
                    v0::Channel::DirectMessage {
                        id,
                        last_message_id,
                        ..
                    }
                    | v0::Channel::Group {
                        id,
                        last_message_id,
                        ..
                    } => (id.clone(), last_message_id.clone()),
                    _ => unreachable!(),
                };

                let (id_b, last_msg_b) = match &*b.read() {
                    v0::Channel::DirectMessage {
                        id,
                        last_message_id,
                        ..
                    }
                    | v0::Channel::Group {
                        id,
                        last_message_id,
                        ..
                    } => (id.clone(), last_message_id.clone()),
                    _ => unreachable!(),
                };

                match (last_msg_a, last_msg_b) {
                    (Some(a), Some(b)) => b.cmp(&a),
                    (Some(a), None) => id_b.cmp(&a),
                    (None, Some(b)) => b.cmp(&id_a),
                    (None, None) => id_b.cmp(&id_a),
                }
            });

            channels
        });

        rect().child(
            rect()
                .spacing(4.)
                .child(
                    rect()
                        .margin((8., 8., 0., 8.))
                        .padding((24., 8.0))
                        .height(Size::px(48.))
                        .font_size(16)
                        .child("Conversations"),
                )
                .child(
                    rect()
                        .padding((0., 8.))
                        .child(
                            dmlist_nav_button(
                                house(),
                                "Home",
                                &*self.selection.read() == &HomeSelection::Welcome,
                            )
                            .on_press({
                                let mut selection = self.selection.clone();

                                move |_| {
                                    *selection.write() = HomeSelection::Welcome;
                                }
                            }),
                        )
                        .child(
                            dmlist_nav_button(
                                users_round(),
                                "Friends",
                                &*self.selection.read() == &HomeSelection::Friends,
                            )
                            .on_press({
                                let mut selection = self.selection.clone();

                                move |_| {
                                    *selection.write() = HomeSelection::Friends;
                                }
                            }),
                        )
                        .child(
                            dmlist_nav_button(
                                notebook_text(),
                                "Saved Notes",
                                saved_messages.read().as_ref().map(|c| c.id())
                                    == self.selection.read().channel_id(),
                            )
                            .on_press({
                                let mut radio = radio.clone();
                                let saved_messages = saved_messages.clone();
                                let mut selection = self.selection.clone();

                                move |_| {
                                    spawn(async move {
                                        let id = if let Some(channel) = &*saved_messages.peek() {
                                            channel.id().to_string()
                                        } else {
                                            let dm = http()
                                                .open_dm(
                                                    &radio.peek_state().user_id.clone().unwrap(),
                                                )
                                                .await
                                                .unwrap();
                                            let id = dm.id().to_string();

                                            radio
                                                .write_channel(AppChannel::Channels)
                                                .channels
                                                .insert(id.clone(), dm);

                                            radio
                                                .write_channel(AppChannel::ChannelMessages)
                                                .channel_messages
                                                .insert(id.clone(), Vec::new());

                                            id
                                        };

                                        *selection.write() = HomeSelection::DM(id);
                                    });
                                }
                            }),
                        ),
                )
                .child(
                    rect()
                        .child(
                            label()
                                .margin((28., 16., 8., 16.))
                                .text("Direct Messages")
                                .font_size(13),
                        )
                        .child(
                            VirtualScrollView::new({
                                let selection = self.selection.clone();
                                move |idx, _| {
                                    let channel = channels.read()[idx].clone();

                                    rect()
                                        .padding((0., 8.))
                                        .key(channel.read().id())
                                        .child(DMButton {
                                            channel,
                                            selection: selection.clone(),
                                        })
                                        .into_element()
                                }
                            })
                            .item_size(42.)
                            .length(channels.read().len()),
                        ),
                ),
        )
    }
}

pub fn dmlist_nav_button(icon: Bytes, title: &'static str, selected: bool) -> Rect {
    rect()
        .horizontal()
        .padding((0., 8., 0., 8.))
        .spacing(8.)
        .height(Size::px(42.))
        .cross_align(Alignment::Center)
        .corner_radius(42.)
        .overflow(Overflow::Clip)
        .font_size(15)
        .child(svg(icon).width(Size::px(24.)).height(Size::px(24.)))
        .child(title)
        .maybe(selected, |btn| btn.background(0xff384379).color(0xffdde1ff))
        .width(Size::Fill)
}
