use freya::radio::{RadioChannel, RadioStation};
use indexmap::IndexMap;
use std::collections::HashMap;
use stoat_database::events::client::EventV1;
use stoat_models::v0::{
    Channel, FieldsChannel, FieldsMessage, FieldsServer, Member, Message, RelationshipStatus,
    Server, User,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connected,
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

#[derive(Debug)]
pub struct AppState {
    pub state: ConnectionState,
    pub selection: Selection,
    pub selected_channel: Option<String>,
    pub user_id: Option<String>,
    pub users: HashMap<String, User>,
    pub servers: IndexMap<String, Server>,
    pub members: HashMap<String, HashMap<String, Member>>,
    pub channels: HashMap<String, Channel>,
    pub channel_messages: HashMap<String, Vec<String>>,
    pub messages: HashMap<String, HashMap<String, Message>>,
    pub settings: Option<SettingsPage>,
}

impl Default for AppState {
    fn default() -> Self {
        let mut this = Self {
            state: Default::default(),
            selection: Default::default(),
            selected_channel: Default::default(),
            user_id: Default::default(),
            users: Default::default(),
            servers: Default::default(),
            members: Default::default(),
            channels: Default::default(),
            channel_messages: Default::default(),
            messages: Default::default(),
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
    SelectedChannel,
    UserId,
    Users,
    Servers,
    Members,
    Channels,
    ChannelMessages,
    Messages,
    Settings,
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
        .insert(channel.id().to_string(), Vec::new());

    station
        .write_channel(AppChannel::Messages)
        .messages
        .insert(channel.id().to_string(), HashMap::new());

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

pub fn update_state(event: EventV1, station: RadioStation<AppState, AppChannel>) {
    match event {
        EventV1::Bulk { v } => {
            for e in v {
                update_state(e, station);
            }
        }
        EventV1::Authenticated => {}
        EventV1::Logout => {}
        EventV1::Pong { data: _ } => {}
        EventV1::Ready {
            users,
            servers,
            channels,
            members,
            emojis: _,
            user_settings: _,
            channel_unreads: _,
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

            // for voice_state in voice_states.into_iter().flatten() {
            //     context.cache.insert_voice_state(voice_state);
            // }

            // for emoji in emojis.into_iter().flatten() {
            //     context.cache.insert_emoji(emoji);
            // }

            set_state(ConnectionState::Connected, station);
        }
        EventV1::Message(message) => {
            insert_message(message.clone(), station);
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
