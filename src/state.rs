use freya::{
    prelude::State,
    radio::{RadioChannel, RadioStation},
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use stoat_database::events::client::EventV1;
use stoat_models::v0::{
    Channel, FieldsChannel, FieldsMessage, FieldsServer, Member, Message, RelationshipStatus,
    Server, User, UserSettings,
};
use stoat_result::ErrorType;

use crate::Config;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connected,
    Reconnecting,
    Reconnected,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Selection {
    #[default]
    Home,
    Server(String),
    Discover,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum SettingsPage {
    #[default]
    Account,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct OrderingSettings {
    pub servers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationState {
    All,
    Mention,
    None,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub struct MuteState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<u128>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct NotificationsSettings {
    pub server: HashMap<String, NotificationState>,
    pub channel: HashMap<String, NotificationState>,
    pub server_mutes: HashMap<String, MuteState>,
    pub channel_mutes: HashMap<String, MuteState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationBadge {
    Unread,
    Mentions(usize),
}

#[derive(Debug, Default)]
pub struct Settings {
    pub ordering: Option<OrderingSettings>,
    pub notifications: Option<NotificationsSettings>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ChannelUnread {
    pub last_id: Option<String>,
    pub mentions: HashSet<String>,
}

#[derive(Debug, Default)]
pub struct Ready {
    pub events: bool,
    pub settings: bool,
}

impl Ready {
    pub fn is_ready(&self) -> bool {
        self.events && self.settings
    }
}

#[derive(Debug)]
pub struct AppState {
    pub state: ConnectionState,
    pub ready: Ready,
    pub selection: Selection,
    pub selected_channel: Option<String>,
    pub user_id: Option<String>,
    pub users: HashMap<String, User>,
    pub servers: HashMap<String, Server>,
    pub members: HashMap<String, HashMap<String, Member>>,
    pub channels: HashMap<String, Channel>,
    pub channel_messages: HashMap<String, Vec<String>>,
    pub channel_unreads: HashMap<String, ChannelUnread>,
    pub messages: HashMap<String, HashMap<String, Message>>,
    pub settings_page: Option<SettingsPage>,
    pub user_profile: Option<String>,
    pub settings: Settings,
}

impl Default for AppState {
    fn default() -> Self {
        let mut this = Self {
            state: Default::default(),
            ready: Default::default(),
            selection: Default::default(),
            selected_channel: Default::default(),
            user_id: Default::default(),
            users: Default::default(),
            servers: Default::default(),
            members: Default::default(),
            channels: Default::default(),
            channel_messages: Default::default(),
            channel_unreads: Default::default(),
            messages: Default::default(),
            settings_page: Default::default(),
            user_profile: Default::default(),
            settings: Default::default(),
        };

        this.users.insert(
            "00000000000000000000000000".to_string(),
            User {
                id: "00000000000000000000000000".to_string(),
                username: "Stoat".to_string(),
                discriminator: "0000".to_string(),
                display_name: None,
                avatar: None,
                relations: Vec::new(),
                badges: 0,
                status: None,
                flags: 0,
                privileged: false,
                bot: None,
                relationship: RelationshipStatus::None,
                online: false,
            },
        );

        this
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum AppChannel {
    State,
    Selection,
    Ready,
    SelectedChannel,
    UserId,
    Users,
    Servers,
    Members,
    Channels,
    ChannelMessages,
    ChannelUnreads,
    Messages,
    SettingsPage,
    Settings(&'static str),
    UserProfile,
}

impl RadioChannel<AppState> for AppChannel {}

type AppStation = RadioStation<AppState, AppChannel>;

macro_rules! set_enum_varient_values {
    ($enum: ident, $key: ident, $value: expr, ($($varient: path),+)) => {
        match $enum {
            $($varient { $key, .. } )|+ => { *$key = $value },
            _ => {}
        }
    };
}

macro_rules! update_enum_partial {
    ($value: ident, $data: expr, $key: ident, ($($varient: path),+)) => {
        if let Some(new_value) = $data.$key {
            set_enum_varient_values!($value, $key, new_value, ($($varient),+))
        }
    };

    (optional, $value: ident, $data: expr, $key: ident, ($($varient: path),+)) => {
        set_enum_varient_values!($value, $key, $data.$key, ($($varient),+))
    };
}

macro_rules! update_multi_enum_partial {
    ($value: ident, $data: expr, ($( $( $(@$optional:tt)? optional )? ($key: ident, ($($varient: path),+))),+ $(,)?)) => {
        $(update_enum_partial!($( $($optional)? optional,)? $value, $data, $key, ($($varient),+)));+
    };
}

pub fn set_state(state: ConnectionState, mut station: AppStation) {
    station.write_channel(AppChannel::State).state = state;
}

pub fn set_current_user_id(user_id: String, mut station: AppStation) {
    station.write_channel(AppChannel::UserId).user_id = Some(user_id);
}

pub fn insert_user(user: User, mut station: AppStation) {
    station
        .write_channel(AppChannel::Users)
        .users
        .insert(user.id.clone(), user);
}

pub fn insert_server(server: Server, mut station: AppStation) {
    station
        .write_channel(AppChannel::Servers)
        .servers
        .insert(server.id.clone(), server);
}

pub fn insert_channel(channel: Channel, mut station: AppStation) {
    station
        .write_channel(AppChannel::ChannelMessages)
        .channel_messages
        .entry(channel.id().to_string())
        .or_default();

    station
        .write_channel(AppChannel::Messages)
        .messages
        .entry(channel.id().to_string())
        .or_default();

    station
        .write_channel(AppChannel::Channels)
        .channels
        .insert(channel.id().to_string(), channel);
}

pub fn insert_message(message: Message, mut station: AppStation) {
    // if let Some(user) = message.user.clone() {
    //     insert_user(user, station);
    // }

    // if let Some(member) = message.member.clone() {
    //     insert_member(member, station);
    // }

    station
        .write_channel(AppChannel::Messages)
        .messages
        .entry(message.channel.clone())
        .or_default()
        .insert(message.id.clone(), message.clone());

    station
        .write_channel(AppChannel::ChannelMessages)
        .channel_messages
        .entry(message.channel.clone())
        .or_default()
        .push(message.id);
}

pub fn insert_member(member: Member, mut station: AppStation) {
    station
        .write_channel(AppChannel::Members)
        .members
        .entry(member.id.server.clone())
        .or_default()
        .insert(member.id.user.clone(), member);
}

pub fn update_server(server_id: &str, mut station: AppStation, f: impl FnOnce(&mut Server)) {
    if let Some(server) = station
        .write_channel(AppChannel::Servers)
        .servers
        .get_mut(server_id)
    {
        f(server)
    }
}

pub fn update_channel(channel_id: &str, mut station: AppStation, f: impl FnOnce(&mut Channel)) {
    if let Some(channel) = station
        .write_channel(AppChannel::Channels)
        .channels
        .get_mut(channel_id)
    {
        f(channel)
    }
}

pub fn update_message(
    message_id: &str,
    channel_id: &str,
    mut station: AppStation,
    f: impl FnOnce(&mut Message),
) {
    if let Some(message) = station
        .write_channel(AppChannel::Messages)
        .messages
        .get_mut(channel_id)
        .and_then(|messages| messages.get_mut(message_id))
    {
        f(message)
    }
}

pub fn delete_message(message_id: &str, channel_id: &str, mut station: AppStation) {
    if let Some(messages) = station
        .write_channel(AppChannel::Messages)
        .messages
        .get_mut(channel_id)
    {
        messages.remove(message_id);
    }

    if let Some(messages) = station
        .write_channel(AppChannel::ChannelMessages)
        .channel_messages
        .get_mut(channel_id)
    {
        messages.retain(|id| id != message_id);
    }
}

pub fn set_selection(selection: Selection, mut station: AppStation) {
    station.write_channel(AppChannel::Selection).selection = selection;
}

pub fn set_selected_channel(channel_id: Option<String>, mut station: AppStation) {
    station
        .write_channel(AppChannel::SelectedChannel)
        .selected_channel = channel_id;
}

pub fn update_settings(settings: UserSettings, mut station: AppStation) {
    for (key, (_ts, payload)) in settings.into_iter() {
        match key.as_str() {
            "ordering" => {
                if let Ok(value) = serde_json::from_str(&payload) {
                    station
                        .write_channel(AppChannel::Settings("ordering"))
                        .settings
                        .ordering = Some(value)
                }
            }
            "notifications" => {
                if let Ok(value) = Ok::<_, ()>(serde_json::from_str(&payload).unwrap()) {
                    station
                        .write_channel(AppChannel::Settings("notifications"))
                        .settings
                        .notifications = Some(value)
                }
            }
            _ => {}
        }
    }
}

pub fn insert_channel_unread(id: String, unread: ChannelUnread, mut station: AppStation) {
    station
        .write_channel(AppChannel::ChannelUnreads)
        .channel_unreads
        .insert(id, unread);
}

pub fn ack_message(channel_id: &str, message_id: String, mut station: AppStation) {
    if let Some(unread) = station
        .write_channel(AppChannel::ChannelUnreads)
        .channel_unreads
        .get_mut(channel_id)
    {
        unread.mentions.retain(|id| id > &message_id);
        unread.last_id = Some(message_id);
    }
}

pub fn update_state(
    event: EventV1,
    mut config: State<Config>,
    mut station: RadioStation<AppState, AppChannel>,
) {
    match event {
        EventV1::Bulk { v } => {
            for e in v {
                update_state(e, config, station);
            }
        }
        EventV1::Authenticated => {}
        EventV1::Logout => {
            config.write().token = None;
        }
        EventV1::Error { data } => match &data.error_type {
            ErrorType::InvalidSession => {
                config.write().token = None;
            }
            _ => {
                log::error!("Error: {data:?}")
            }
        },
        EventV1::Pong { data: _ } => {}
        EventV1::Ready {
            users,
            servers,
            channels,
            members,
            emojis: _,
            user_settings,
            channel_unreads,
            policy_changes: _,
            voice_states: _,
        } => {
            for user in users.into_iter().flatten() {
                if user.relationship == RelationshipStatus::User {
                    set_current_user_id(user.id.clone(), station);
                };

                insert_user(user, station);
            }

            for server in servers.into_iter().flatten() {
                insert_server(server, station);
            }

            for channel in channels.into_iter().flatten() {
                insert_channel(channel, station);
            }

            for member in members.into_iter().flatten() {
                insert_member(member, station);
            }

            for channel_unread in channel_unreads.into_iter().flatten() {
                insert_channel_unread(
                    channel_unread.id.channel,
                    ChannelUnread {
                        last_id: channel_unread.last_id,
                        mentions: channel_unread.mentions.into_iter().collect(),
                    },
                    station,
                );
            }

            if let Some(settings) = user_settings {
                update_settings(settings, station);
            }

            // for voice_state in voice_states.into_iter().flatten() {
            //     context.cache.insert_voice_state(voice_state);
            // }

            // for emoji in emojis.into_iter().flatten() {
            //     context.cache.insert_emoji(emoji);
            // }

            {
                let mut state = station.write_channel(AppChannel::State);

                state.state = if state.state == ConnectionState::Reconnecting && state.ready.events
                {
                    ConnectionState::Reconnected
                } else {
                    ConnectionState::Connected
                };
            }

            station.write_channel(AppChannel::Ready).ready.events = true;
        }
        EventV1::Message(message) => {
            insert_message(message.clone(), station);

            update_channel(&message.channel, station, |channel| {
                if let Channel::TextChannel {
                    last_message_id, ..
                }
                | Channel::Group {
                    last_message_id, ..
                }
                | Channel::DirectMessage {
                    last_message_id, ..
                } = channel
                {
                    *last_message_id = Some(message.id.clone());
                }
            });

            if message
                .mentions
                .as_ref()
                .is_some_and(|m| m.contains(station.peek().user_id.as_ref().unwrap()))
            {
                station
                    .write_channel(AppChannel::ChannelUnreads)
                    .channel_unreads
                    .entry(message.channel.clone())
                    .or_default()
                    .mentions
                    .insert(message.id);
            }
        }
        EventV1::ServerUpdate { id, data, clear } => {
            update_server(&id, station, |server| {
                server.apply_options(data);

                for field in &clear {
                    match field {
                        FieldsServer::Description => server.description = None,
                        FieldsServer::Categories => server.categories = None,
                        FieldsServer::SystemMessages => server.system_messages = None,
                        FieldsServer::Icon => server.icon = None,
                        FieldsServer::Banner => server.banner = None,
                    }
                }
            });
        }
        EventV1::ChannelUpdate { id, data, clear } => {
            update_channel(&id, station, |channel| {
                update_multi_enum_partial!(
                    channel,
                    data.clone(),
                    (
                        (name, (Channel::TextChannel)),
                        (owner, (Channel::Group)),
                        optional(description, (Channel::Group, Channel::TextChannel)),
                        optional(icon, (Channel::Group, Channel::TextChannel)),
                        (nsfw, (Channel::Group, Channel::TextChannel)),
                        (active, (Channel::DirectMessage)),
                        optional(permissions, (Channel::Group)),
                        (role_permissions, (Channel::TextChannel)),
                        optional(default_permissions, (Channel::TextChannel)),
                        optional(
                            last_message_id,
                            (Channel::DirectMessage, Channel::Group, Channel::TextChannel)
                        )
                    )
                );

                for field in &clear {
                    match field {
                        FieldsChannel::Description => set_enum_varient_values!(
                            channel,
                            description,
                            None,
                            (Channel::Group, Channel::TextChannel)
                        ),
                        FieldsChannel::Icon => set_enum_varient_values!(
                            channel,
                            icon,
                            None,
                            (Channel::Group, Channel::TextChannel)
                        ),
                        FieldsChannel::DefaultPermissions => set_enum_varient_values!(
                            channel,
                            default_permissions,
                            None,
                            (Channel::TextChannel)
                        ),
                        FieldsChannel::Voice => {
                            set_enum_varient_values!(channel, voice, None, (Channel::TextChannel))
                        }
                    }
                }
            });
        }
        EventV1::MessageUpdate {
            id,
            channel,
            data,
            clear,
        } => {
            update_message(&id, &channel, station, |message| {
                message.apply_options(data.clone());

                for field in &clear {
                    match field {
                        FieldsMessage::Pinned => message.pinned = None,
                    }
                }
            });
        }
        EventV1::MessageDelete { id, channel } => {
            delete_message(&id, &channel, station);
        }
        EventV1::MessageReact {
            id,
            channel_id,
            user_id,
            emoji_id,
        } => {
            update_message(&id, &channel_id, station, |message| {
                message
                    .reactions
                    .entry(emoji_id.clone())
                    .or_default()
                    .insert(user_id.clone());
            });
        }
        EventV1::MessageUnreact {
            id,
            channel_id,
            user_id,
            emoji_id,
        } => {
            update_message(&id, &channel_id, station, |message| {
                if let Some(users) = message.reactions.get_mut(&emoji_id) {
                    users.remove(&user_id);

                    if users.is_empty() {
                        message.reactions.remove(&emoji_id);
                    };
                }
            });
        }
        EventV1::MessageRemoveReaction {
            id,
            channel_id,
            emoji_id,
        } => {
            update_message(&id, &channel_id, station, |message| {
                message.reactions.remove(&emoji_id);
            });
        }
        EventV1::ChannelCreate(channel) => {
            insert_channel(channel, station);
        }
        EventV1::UserSettingsUpdate { id: _id, update } => {
            update_settings(update, station);
        }
        EventV1::ChannelAck {
            id,
            user: _,
            message_id,
        } => {
            ack_message(&id, message_id, station);
        }
        // EventV1::MessageAppend {
        //     id,
        //     channel: _,
        //     append,
        // } => {
        //     if let Some(message) = context.cache.update_message_with(&id, |message| {
        //         if let Some(embeds) = append.embeds.clone() {
        //             message.embeds.get_or_insert_default().extend(embeds);
        //         }

        //         message.clone()
        //     }) {}
        // }
        // EventV1::ChannelStartTyping { id, user } => {
        //     context
        //         .notifiers
        //         .invoke_typing_start_waiters(&(id.clone(), user.clone()))
        //         .await;

        //     handle_event!(handler, context, typing_start, (id, user))
        // }
        // EventV1::ChannelStopTyping { id, user } => {
        //     context
        //         .notifiers
        //         .invoke_typing_stop_waiters(&(id.clone(), user.clone()))
        //         .await;

        //     handle_event!(handler, context, typing_stop, (id, user))
        // }
        _ => {}
    }
}
