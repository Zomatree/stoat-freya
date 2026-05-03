use std::{borrow::Cow, collections::HashMap};

use freya::{
    icons::lucide::{chevron_left, hash, pin, users_round},
    prelude::*,
    radio::use_radio,
};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{
        ChannelMessages, MemberList, MessageAttachmentsPreview, MessageModel, MessageReplyPreview,
        StoatButton, Textbox,
    },
    map_readable, use_config,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ReplyIntent {
    pub message: MessageModel,
    pub mention: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub struct ReplyController(State<Vec<ReplyIntent>>);

impl ReplyController {
    pub fn get_replies(&self) -> impl Iterator<Item = Readable<ReplyIntent>> {
        self.0.read().clone().into_iter().map(|reply| {
            let id = reply.message.message.peek().id.clone();

            map_readable::<Vec<ReplyIntent>, ReplyIntent>(self.0.into_readable(), move |replies| {
                replies
                    .iter()
                    .find(|r| r.message.message.peek().id == id)
                    .unwrap()
            })
        })
    }

    pub fn toggle_mention(&mut self, message_id: &str) {
        if let Some(reply) = self
            .0
            .write()
            .iter_mut()
            .find(|r| r.message.message.peek().id == message_id)
        {
            reply.mention = !reply.mention;
        }
    }

    pub fn add_reply(&mut self, message: MessageModel, mention: bool) {
        let message_id = &message.message.peek().id;
        let mut replies = self.0.write();

        if replies
            .iter()
            .any(|reply| &reply.message.message.peek().id == message_id)
        {
            return;
        };

        replies.push(ReplyIntent { message, mention });
    }

    pub fn remove_reply(&mut self, message_id: &str) {
        self.0.with_mut(|mut replies| {
            replies.retain(|r| r.message.message.peek().id != message_id);
        });
    }

    pub fn take_replies(&mut self) -> Vec<v0::ReplyIntent> {
        let replies = std::mem::take(&mut *self.0.write());

        replies
            .into_iter()
            .map(|reply| v0::ReplyIntent {
                id: reply.message.message.peek().id.clone(),
                mention: reply.mention,
                fail_if_not_exists: Some(true),
            })
            .collect()
    }
}

#[derive(PartialEq)]
pub struct Channel {
    pub channel: Readable<v0::Channel>,
    pub server: Option<Readable<v0::Server>>,
}

impl Component for Channel {
    fn render(&self) -> impl IntoElement {
        let mut config = use_config();
        let radio = use_radio(AppChannel::ChannelMessages);

        let channel = self.channel.clone();
        let channel_messages = radio
            .slice_current(move |state| state.channel_messages.get(channel.read().id()).unwrap());

        let mut textbox_size = use_state(Area::default);
        let textbox_height = textbox_size.read().height();

        let replies = ReplyController(use_state(Vec::<ReplyIntent>::new));
        let attachments = use_state(HashMap::new);

        let hide_channel_list = config.read().hide_channel_list;
        let hide_members_list = config.read().hide_members_list;
        let is_dm = self.server.is_none();

        let search = use_state(String::new);

        rect()
            .child(
                rect()
                    .horizontal()
                    .height(Size::px(48.))
                    .padding((0., 24., 0., 16.))
                    .margin((8., 0.))
                    .spacing(10.)
                    .cross_align(Alignment::Center)
                    .content(Content::Flex)
                    .child(
                        StoatButton::new()
                            .on_press(move |_| {
                                config.write().hide_channel_list = !hide_channel_list;
                            })
                            .child(
                                rect()
                                    .cross_align(Alignment::Center)
                                    .horizontal()
                                    .child(
                                        svg(chevron_left())
                                            .width(Size::px(20.))
                                            .height(Size::px(20.))
                                            .rotate(if hide_channel_list { 180. } else { 0. }),
                                    )
                                    .child(svg(hash()).width(Size::px(24.)).height(Size::px(24.))),
                            ),
                    )
                    .child(
                        label()
                            .text(match &*self.channel.read() {
                                v0::Channel::DirectMessage { recipients, .. } => {
                                    let user_id = radio.peek_state().user_id.clone().unwrap();

                                    let other = recipients
                                        .iter()
                                        .find(|&id| id != &*user_id)
                                        .unwrap()
                                        .clone();

                                    let user = radio.slice(AppChannel::Users, move |state| {
                                        state.users.get(&other).unwrap()
                                    });

                                    Cow::Owned(user.read().username.clone())
                                }
                                v0::Channel::Group { name, .. }
                                | v0::Channel::TextChannel { name, .. } => Cow::Owned(name.clone()),
                                v0::Channel::SavedMessages { .. } => {
                                    Cow::Borrowed("Saved Messages")
                                }
                            })
                            .font_size(16),
                    )
                    .child(rect().width(Size::flex(1.)))
                    .child(
                        StoatButton::new().on_press(move |_| {}).child(
                            rect()
                                .horizontal()
                                .height(Size::px(40.))
                                .padding((0., 8.))
                                .center()
                                .child(svg(pin()).width(Size::px(24.)).height(Size::px(24.))),
                        ),
                    )
                    .child(
                        StoatButton::new()
                            .on_press(move |_| {
                                config.write().hide_members_list = !hide_members_list
                            })
                            .child(
                                rect()
                                    .horizontal()
                                    .height(Size::px(40.))
                                    .padding((0., 8.))
                                    .center()
                                    .child(
                                        svg(users_round())
                                            .width(Size::px(24.))
                                            .height(Size::px(24.)),
                                    ),
                            ),
                    )
                    .child(
                        Input::new(search)
                            .placeholder("Search messages...")
                            .border_fill(Color::TRANSPARENT)
                            .inner_margin((10., 16.))
                            .corner_radius(40.)
                            .width(Size::px(240.))
                            .background(0xff292a2f),
                    ),
            )
            .child(
                rect()
                    .horizontal()
                    .child(
                        rect()
                            .margin((0., 8., 8., 8.))
                            .width(Size::func_data(
                                move |size| {
                                    Some(
                                        size.parent
                                            - if is_dm || hide_members_list { 0. } else { 248. },
                                    )
                                },
                                &(is_dm || hide_members_list),
                            ))
                            .corner_radius(28.)
                            .background(0xff0d0e13)
                            .overflow(Overflow::Clip)
                            .child(
                                rect()
                                    .height(Size::func_data(
                                        move |size| Some(size.parent - (textbox_height + 8.)),
                                        &(textbox_height as i32),
                                    ))
                                    .child(ChannelMessages {
                                        replies,
                                        channel: self.channel.clone(),
                                        channel_messages: channel_messages.into_readable(),
                                        server: self.server.clone(),
                                    }),
                            )
                            .child(
                                rect()
                                    .maybe_child((!attachments.read().is_empty()).then(|| {
                                        rect()
                                            .margin((0., 0., 8., 0.))
                                            .width(Size::func(|size| Some(size.parent - 16.)))
                                            .child(MessageAttachmentsPreview {
                                                attachments: attachments.clone(),
                                            })
                                            .into_element()
                                    }))
                                    .child(rect().children(replies.get_replies().map(|reply| {
                                        rect()
                                            .key(&reply.read().message.message.peek().id)
                                            .margin((0., 0., 8., 0.))
                                            .child(MessageReplyPreview {
                                                replies,
                                                reply,
                                                channel: self.channel.clone(),
                                            })
                                            .into_element()
                                    })))
                                    .child(Textbox {
                                        replies,
                                        attachments,
                                        channel: self.channel.clone(),
                                    })
                                    .margin((0., 8., 8., 8.))
                                    .on_sized(move |e: Event<SizedEventData>| {
                                        textbox_size.set(e.area)
                                    }),
                            ),
                    )
                    .maybe_child(self.server.as_ref().filter(|_| !hide_members_list).map(
                        |server| {
                            rect()
                                .child(MemberList {
                                    server: server.clone(),
                                })
                                .min_width(Size::px(240.))
                        },
                    )),
            )
    }

    fn render_key(&self) -> DiffKey {
        (&self.channel.peek().id().to_string()).into()
    }
}
