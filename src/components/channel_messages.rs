use std::{collections::HashMap, fmt::Debug};

use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{MessageGroup, ReplyController},
    http, map_readable,
};

#[derive(PartialEq)]
pub struct ChannelMessages {
    pub replies: ReplyController,
    pub channel: Readable<v0::Channel>,
    pub server: Option<Readable<v0::Server>>,
    pub channel_messages: Readable<Vec<String>>,
}

#[derive(Clone, PartialEq)]
pub struct MessageModel {
    pub message: Readable<v0::Message>,
    pub user: Readable<v0::User>,
    pub member: Option<Readable<v0::Member>>,
    pub replies: Vec<MessageModel>,
}

impl Debug for MessageModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageModel")
            .field("message", &self.message.peek())
            .field("user", &self.user.peek())
            .field("member", &self.member.as_ref().map(|m| m.peek()))
            .field("replies", &self.replies)
            .finish()
    }
}

impl Component for ChannelMessages {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Messages);
        let mut user_radio = use_radio(AppChannel::Users);
        let mut member_radio = use_radio(AppChannel::Members);

        let messages_models = use_state(Vec::<MessageModel>::new);

        {
            let channel_messages = self.channel_messages.clone();
            let channel = self.channel.clone();
            let server = self.server.clone();

            use_side_effect(move || {
                let channel_messages = channel_messages.clone();
                let channel = channel.clone();
                let server = server.clone();
                let mut messages = messages_models.clone();
                let mut radio = radio.clone();

                drop(channel_messages.read());

                spawn(async move {
                    let mut message_models = Vec::new();
                    let channel_id = channel.read().id().to_string();
                    let channel_messages = channel_messages.read().clone();

                    for message_id in channel_messages {
                        let message_readable: Readable<v0::Message> = radio
                            .slice_current({
                                let channel_id = channel_id.clone();
                                move |state| {
                                    state
                                        .messages
                                        .get(&channel_id)
                                        .unwrap()
                                        .get(&message_id)
                                        .unwrap()
                                }
                            })
                            .into_readable();

                        let message = message_readable.read().clone();

                        let user = if message.user.is_some() {
                            map_readable(message_readable.clone(), |message| {
                                message.user.as_ref().unwrap()
                            })
                        } else {
                            let exists = user_radio.read().users.contains_key(&message.author);

                            if !exists {
                                let user = http().fetch_user(&message.author).await.unwrap();

                                user_radio.write().users.insert(user.id.clone(), user);
                            };

                            user_radio
                                .slice_current({
                                    let user_id = message.author.clone();
                                    move |state| state.users.get(&user_id).unwrap()
                                })
                                .into_readable()
                        };

                        let member = if message.member.is_some() {
                            Some(map_readable(message_readable.clone(), |message| {
                                message.member.as_ref().unwrap()
                            }))
                        } else if message.author.as_str() != "00000000000000000000000000"
                            && let Some(server) = server.clone()
                        {
                            let exists = member_radio
                                .read()
                                .members
                                .get(&server.read().id)
                                .is_some_and(|members| members.contains_key(&message.author));

                            let fetched = if !exists {
                                let server_id = server.read().id.clone();

                                if let Some(member) = http()
                                    .fetch_member(&server_id, &message.author.clone())
                                    .await
                                    .ok()
                                {
                                    member_radio
                                        .write()
                                        .members
                                        .entry(member.id.server.clone())
                                        .or_default()
                                        .insert(member.id.user.clone(), member);

                                    true
                                } else {
                                    false
                                }
                            } else {
                                true
                            };

                            if fetched {
                                let members: Readable<HashMap<String, v0::Member>> = member_radio
                                    .slice_current(move |state| {
                                        state.members.get(&server.read().id).unwrap()
                                    })
                                    .into_readable();

                                Some(map_readable(members, {
                                    let user_id = message.author.clone();
                                    move |members| members.get(&user_id).unwrap()
                                }))
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        let mut replies = Vec::new();

                        if let Some(reply_ids) = message.replies.clone() {
                            for reply_id in reply_ids {
                                let exists = radio
                                    .read()
                                    .messages
                                    .get(&message.channel)
                                    .is_some_and(|messages| messages.contains_key(&reply_id));

                                if !exists {
                                    // TODO: handle deleted messages
                                    let reply =
                                        http().fetch_message(&channel_id, &reply_id).await.unwrap();

                                    radio
                                        .write()
                                        .messages
                                        .get_mut(&message.channel)
                                        .unwrap()
                                        .insert(reply_id.clone(), reply);
                                };

                                let message_readable: Readable<v0::Message> = radio
                                    .slice_current({
                                        let channel_id = channel_id.clone();
                                        move |state| {
                                            state
                                                .messages
                                                .get(&channel_id)
                                                .unwrap()
                                                .get(&reply_id)
                                                .unwrap()
                                        }
                                    })
                                    .into_readable();

                                let message = message_readable.read().clone();

                                let user = if message.user.is_some() {
                                    map_readable(message_readable.clone(), |message| {
                                        message.user.as_ref().unwrap()
                                    })
                                } else {
                                    let exists =
                                        user_radio.read().users.contains_key(&message.author);

                                    if !exists {
                                        let user =
                                            http().fetch_user(&message.author).await.unwrap();

                                        user_radio.write().users.insert(user.id.clone(), user);
                                    };

                                    user_radio
                                        .slice_current({
                                            let user_id = message.author.clone();
                                            move |state| state.users.get(&user_id).unwrap()
                                        })
                                        .into_readable()
                                };

                                let member = if message.member.is_some() {
                                    Some(map_readable(message_readable.clone(), |message| {
                                        message.member.as_ref().unwrap()
                                    }))
                                } else if let Some(server) = server.clone() {
                                    let server_id = server.read().id.clone();

                                    let exists =
                                        member_radio.read().members.get(&server_id).is_some_and(
                                            |members| members.contains_key(&message.author),
                                        );

                                    if !exists {
                                        if let Some(member) = http()
                                            .fetch_member(&server_id, &message.author)
                                            .await
                                            .ok()
                                        {
                                            member_radio
                                                .write()
                                                .members
                                                .entry(member.id.server.clone())
                                                .or_default()
                                                .insert(member.id.user.clone(), member);
                                        }
                                    };

                                    let members: Readable<HashMap<String, v0::Member>> =
                                        member_radio
                                            .slice_current(move |state| {
                                                state.members.get(&server.read().id).unwrap()
                                            })
                                            .into_readable();

                                    Some(map_readable(members, {
                                        let user_id = message.author.clone();
                                        move |members| members.get(&user_id).unwrap()
                                    }))
                                } else {
                                    None
                                };

                                replies.push(MessageModel {
                                    message: message_readable,
                                    user,
                                    member,
                                    replies: Vec::new(),
                                });
                            }
                        }

                        message_models.push(MessageModel {
                            message: message_readable,
                            user,
                            member,
                            replies,
                        });
                    }

                    messages.set(message_models);
                });
            });
        }

        let groups = {
            let messages = messages_models.clone();

            use_memo(move || {
                let mut groups: Vec<Vec<MessageModel>> = Vec::new();

                for model in messages.read().iter().cloned() {
                    if let Some(group) = groups.last_mut()
                        && group.first().unwrap().user.peek().id == model.user.peek().id
                        && model.replies.is_empty()
                    {
                        group.push(model);
                    } else {
                        groups.push(vec![model]);
                    }
                }

                groups
            })
        };

        use_future({
            let groups = groups.clone();
            let channel = self.channel.clone();
            let radio = radio.clone();

            move || {
                let groups = groups.clone();
                let channel = channel.clone();
                let mut radio = radio.clone();

                async move {
                    let is_empty = groups.read().is_empty();

                    if is_empty {
                        let channel = channel.read().id().to_string();

                        let v0::BulkMessageResponse::JustMessages(mut messages) = http()
                            .fetch_messages(
                                &channel,
                                &v0::OptionsQueryMessages {
                                    limit: Some(20),
                                    before: None,
                                    after: None,
                                    sort: None,
                                    nearby: None,
                                    include_users: None,
                                },
                            )
                            .await
                            .unwrap()
                        else {
                            panic!()
                        };

                        messages.sort_by(|a, b| a.id.cmp(&b.id));

                        for message in messages {
                            radio
                                .write_channel(AppChannel::ChannelMessages)
                                .channel_messages
                                .get_mut(&channel)
                                .unwrap()
                                .push(message.id.clone());
                            radio
                                .write_channel(AppChannel::Messages)
                                .messages
                                .get_mut(&channel)
                                .unwrap()
                                .insert(message.id.clone(), message.clone());
                        }
                    };
                }
            }
        });

        rect().padding((0., 8.)).child(
            ScrollView::new().child(rect().padding((16., 0., 26., 0.)).spacing(12.).children(
                groups.read().iter().cloned().map(|messages| {
                    rect()
                        .key(format!(
                            "{}{}",
                            messages.first().unwrap().message.peek().id,
                            messages.len()
                        ))
                        .child(MessageGroup {
                            replies: self.replies.clone(),
                            messages,
                            channel: self.channel.clone(),
                        })
                        .into_element()
                }),
            )),
        )
    }
}
