use std::{collections::HashMap, fmt::Debug};

use freya::{prelude::*, radio::use_radio};
use jiff::{Timestamp, tz::TimeZone};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{Message, MessageActions, MessageGroup, ReplyController, TrailingMessage},
    http, map_readable,
    types::Tag,
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
    pub replies: Vec<(String, Option<MessageModel>)>,
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
        let task_running = use_state(|| None::<TaskHandle>);

        {
            let channel_messages = self.channel_messages.clone();
            let channel = self.channel.clone();
            let server = self.server.clone();
            let mut task_running = task_running.clone();

            use_side_effect(move || {
                task_running.take().inspect(|t| t.cancel());

                let channel_messages = channel_messages.clone();
                let channel = channel.clone();
                let server = server.clone();
                let mut messages = messages_models.clone();
                let mut radio = radio.clone();

                drop(channel_messages.read());

                task_running.set(Some(spawn(async move {
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
                                let user = if let Some(webhook) = &message.webhook {
                                    v0::User {
                                        id: message.author.clone(),
                                        username: webhook.name.clone(),
                                        discriminator: "0000".to_string(),
                                        display_name: None,
                                        avatar: webhook.avatar.clone().map(|id| v0::File {
                                            id,
                                            tag: Tag::Avatars.into(),
                                            filename: String::new(),
                                            metadata: v0::Metadata::File,
                                            content_type: String::new(),
                                            size: 0,
                                            deleted: None,
                                            reported: None,
                                            message_id: None,
                                            user_id: None,
                                            server_id: None,
                                            object_id: None,
                                        }),
                                        relations: Vec::new(),
                                        badges: 0,
                                        status: None,
                                        flags: 0,
                                        privileged: false,
                                        bot: None,
                                        relationship: v0::RelationshipStatus::None,
                                        online: false,
                                    }
                                } else if let Ok(user) = http().fetch_user(&message.author).await {
                                    user
                                } else {
                                    v0::User {
                                        id: message.author.clone(),
                                        username: format!("Unknown User {}", message.author),
                                        discriminator: "0000".to_string(),
                                        display_name: None,
                                        avatar: None,
                                        relations: Vec::new(),
                                        badges: 0,
                                        status: None,
                                        flags: 0,
                                        privileged: false,
                                        bot: None,
                                        relationship: v0::RelationshipStatus::None,
                                        online: false,
                                    }
                                };

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
                            && message.webhook.is_none()
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
                                    if let Ok(reply) =
                                        http().fetch_message(&channel_id, &reply_id).await
                                    {
                                        radio
                                            .write()
                                            .messages
                                            .get_mut(&message.channel)
                                            .unwrap()
                                            .insert(reply_id.clone(), reply);
                                    } else {
                                        replies.push((reply_id, None));
                                        continue;
                                    }
                                };

                                let message_readable: Readable<v0::Message> = radio
                                    .slice_current({
                                        let channel_id = channel_id.clone();
                                        let reply_id = reply_id.clone();

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
                                        let user = if let Some(webhook) = &message.webhook {
                                            v0::User {
                                                id: message.author.clone(),
                                                username: webhook.name.clone(),
                                                discriminator: "0000".to_string(),
                                                display_name: None,
                                                avatar: webhook.avatar.clone().map(|id| v0::File {
                                                    id,
                                                    tag: Tag::Avatars.into(),
                                                    filename: String::new(),
                                                    metadata: v0::Metadata::File,
                                                    content_type: String::new(),
                                                    size: 0,
                                                    deleted: None,
                                                    reported: None,
                                                    message_id: None,
                                                    user_id: None,
                                                    server_id: None,
                                                    object_id: None,
                                                }),
                                                relations: Vec::new(),
                                                badges: 0,
                                                status: None,
                                                flags: 0,
                                                privileged: false,
                                                bot: None,
                                                relationship: v0::RelationshipStatus::None,
                                                online: false,
                                            }
                                        } else if let Ok(user) =
                                            http().fetch_user(&message.author).await
                                        {
                                            user
                                        } else {
                                            v0::User {
                                                id: message.author.clone(),
                                                username: format!(
                                                    "Unknown User {}",
                                                    message.author
                                                ),
                                                discriminator: "0000".to_string(),
                                                display_name: None,
                                                avatar: None,
                                                relations: Vec::new(),
                                                badges: 0,
                                                status: None,
                                                flags: 0,
                                                privileged: false,
                                                bot: None,
                                                relationship: v0::RelationshipStatus::None,
                                                online: false,
                                            }
                                        };

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
                                    && message.webhook.is_none()
                                    && let Some(server) = server.clone()
                                {
                                    let server_id = server.read().id.clone();

                                    let exists =
                                        member_radio.read().members.get(&server_id).is_some_and(
                                            |members| members.contains_key(&message.author),
                                        );

                                    let fetched = if !exists {
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

                                            true
                                        } else {
                                            false
                                        }
                                    } else {
                                        true
                                    };

                                    if fetched {
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
                                    }
                                } else {
                                    None
                                };

                                replies.push((
                                    reply_id,
                                    Some(MessageModel {
                                        message: message_readable,
                                        user,
                                        member,
                                        replies: Vec::new(),
                                    }),
                                ));
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
                })));
            });
        }

        let groups = {
            let messages = messages_models.clone();

            use_memo(move || {
                let mut groups: Vec<Vec<MessageModel>> = Vec::new();

                for model in messages.read().iter().cloned() {
                    if let Some(group) = groups.last_mut() {
                        let last = group.last().unwrap();

                        if last.user.peek().id != model.user.peek().id || !model.replies.is_empty()
                        {
                            groups.push(vec![model]);
                            continue;
                        };

                        let last_datetime = Timestamp::try_from(
                            ulid::Ulid::from_string(&last.message.peek().id)
                                .unwrap()
                                .datetime(),
                        )
                        .unwrap()
                        .to_zoned(TimeZone::system());

                        let current_datetime = Timestamp::try_from(
                            ulid::Ulid::from_string(&model.message.peek().id)
                                .unwrap()
                                .datetime(),
                        )
                        .unwrap()
                        .to_zoned(TimeZone::system());

                        let diff = (current_datetime.timestamp().as_second()
                            - last_datetime.timestamp().as_second())
                        .abs();

                        if last_datetime.date() != current_datetime.date() || diff >= 420 {
                            groups.push(vec![model]);
                            continue;
                        }

                        let last_msg = last.message.read();

                        if last_msg.system.is_some() {
                            groups.push(vec![model]);
                            continue;
                        }

                        let current_msg = model.message.read();

                        if current_msg.system.is_some()
                            || current_msg.masquerade != last_msg.masquerade
                        {
                            groups.push(vec![model]);
                            continue;
                        }

                        group.push(model);
                    } else {
                        groups.push(vec![model]);
                    }
                }

                groups
            })
        };

        use_future({
            let channel_messages = self.channel_messages.clone();
            let channel = self.channel.clone();
            let radio = radio.clone();
            let server = self.server.clone();

            move || {
                let channel_messages = channel_messages.clone();
                let channel = channel.clone();
                let mut radio = radio.clone();
                let server = server.clone();

                async move {
                    let is_empty = channel_messages.read().is_empty();

                    if is_empty {
                        let channel = channel.read().id().to_string();

                        let v0::BulkMessageResponse::MessagesAndUsers {
                            mut messages,
                            users,
                            members,
                        } = http()
                            .fetch_messages(
                                &channel,
                                &v0::OptionsQueryMessages {
                                    limit: Some(10),
                                    before: None,
                                    after: None,
                                    sort: None,
                                    nearby: None,
                                    include_users: Some(true),
                                },
                            )
                            .await
                            .unwrap()
                        else {
                            panic!()
                        };

                        {
                            let mut state = radio.write_channel(AppChannel::Users);

                            for user in users {
                                state.users.entry(user.id.clone()).or_insert(user);
                            }
                        }

                        {
                            if let Some(members) = members
                                && let Some(server_id) = server.map(|s| s.read().id.clone())
                            {
                                let mut state = radio.write_channel(AppChannel::Members);

                                let server_members = state.members.get_mut(&server_id).unwrap();

                                for member in members {
                                    server_members
                                        .entry(member.id.user.clone())
                                        .or_insert(member);
                                }
                            }
                        }

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
            ScrollView::new().child(
                rect().padding((16., 0., 26., 0.)).children(
                    groups
                        .read()
                        .iter()
                        .cloned()
                        .map(|messages| {
                            let first = messages.first().unwrap();

                            let mut elements = vec![
                                MessageActions::new(
                                    self.replies,
                                    self.channel.clone(),
                                    first.clone(),
                                )
                                .margin((12., 16., 0., 0.))
                                .padding((2., 0.))
                                .child(Message {
                                    channel: self.channel.clone(),
                                    message: first.clone(),
                                })
                                .into_element(),
                            ];

                            for message in &messages[1..] {
                                elements.push(
                                    MessageActions::new(
                                        self.replies,
                                        self.channel.clone(),
                                        first.clone(),
                                    )
                                    .margin((0., 16., 0., 0.))
                                    .padding((2., 0.))
                                    .child(TrailingMessage {
                                        channel: self.channel.clone(),
                                        message: message.clone(),
                                    })
                                    .into_element(),
                                );
                            }
                            elements
                        })
                        .flatten(),
                ),
            ),
        )
    }
}
