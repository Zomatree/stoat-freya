use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, Config, NotificationBadge, Selection, components::server_icon, get_unread_badge,
    is_server_muted,
};

#[derive(PartialEq)]
pub struct ServerListButton {
    pub server: Readable<v0::Server>,
}

impl Component for ServerListButton {
    fn render(&self) -> impl IntoElement {
        let config = use_consume::<State<Config>>();
        let mut radio = use_radio(AppChannel::Servers);

        let selection = radio.slice(AppChannel::Selection, |state| &state.selection);
        let notifications_settings = radio.slice(AppChannel::Settings("notifications"), |state| {
            &state.settings.notifications
        });
        let unreads = radio.slice(AppChannel::ChannelUnreads, |state| &state.channel_unreads);
        let channels = radio.slice(AppChannel::Channels, |state| &state.channels);

        let server = use_memo({
            let server = self.server.clone();
            move || server.read().clone()
        });

        let muted = use_memo({
            let notifications_settings = notifications_settings.clone().into_readable();
            let server = server.clone();

            move || is_server_muted(&server.peek().id, notifications_settings.clone())
        });

        let badge = use_memo({
            let server = server.clone();

            move || {
                if !*muted.read() {
                    let badges = server
                        .read()
                        .channels
                        .iter()
                        .filter_map(|id| {
                            if let Some(unread) = unreads.read().get(id) {
                                Some((id.clone(), unread.clone()))
                            } else {
                                None
                            }
                        })
                        .filter_map(|(id, unread)| {
                            channels.read().get(&id).and_then(|channel| get_unread_badge(channel, &unread))
                        })
                        .collect::<Vec<_>>();

                    let mut mention_count = 0;
                    let mut has_unread = false;

                    for badge in badges {
                        match badge {
                            NotificationBadge::Unread => has_unread = true,
                            NotificationBadge::Mentions(count) => mention_count += count,
                        }
                    }

                    if mention_count > 0 {
                        Some(NotificationBadge::Mentions(mention_count))
                    } else if has_unread {
                        Some(NotificationBadge::Unread)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        });

        let selected =
            use_memo(move || &*selection.read() == &Selection::Server(server.peek().id.clone()));

        rect()
            .horizontal()
            .maybe_child(
                (*selected.read() || *badge.read() == Some(NotificationBadge::Unread)).then(|| {
                    rect()
                        .width(Size::px(12.))
                        .height(Size::px(if *selected.read() { 32. } else { 8. }))
                        .layer(Layer::RelativeOverlay(1))
                        .position(
                            Position::new_absolute()
                                .left(-16.)
                                .top(if *selected.read() { 5. } else { 17. }),
                        )
                        .corner_radius(4.)
                        .background(0xffe3e1e9)
                }),
            )
            .child(
                rect()
                    .width(Size::px(42.0))
                    .height(Size::px(42.0))
                    .corner_radius(42.)
                    .overflow(Overflow::Clip)
                    .child(server_icon(&server.read()))
                    .on_press({
                        move |_| {
                            radio.write_channel(AppChannel::Selection).selection =
                                Selection::Server(server.peek().id.clone());

                            let channel_id = config
                                .read()
                                .last_channels
                                .get(&server.peek().id)
                                .cloned()
                                .or_else(|| {
                                    let channels =
                                        radio.slice(AppChannel::Channels, |state| &state.channels);

                                    server
                                        .read()
                                        .channels
                                        .iter()
                                        .find(|&id| channels.read().contains_key(id))
                                        .cloned()
                                });

                            radio
                                .write_channel(AppChannel::SelectedChannel)
                                .selected_channel = channel_id;
                        }
                    })
                    .on_pointer_enter(|_| {
                        Cursor::set(CursorIcon::Pointer);
                    })
                    .on_pointer_leave(|_| {
                        Cursor::set(CursorIcon::Default);
                    }),
            )
            .maybe_child(
                badge
                    .read()
                    .as_ref()
                    .and_then(|badge| {
                        if let NotificationBadge::Mentions(count) = badge {
                            Some(count)
                        } else {
                            None
                        }
                    })
                    .map(|count| {
                        rect()
                            .position(Position::new_absolute().left(31.))
                            .layer(Layer::Overlay)
                            .width(Size::px(12.))
                            .height(Size::px(12.))
                            .corner_radius(12.)
                            .center()
                            .background(0xffffb4ab)
                            .color(0xff690005)
                            .font_size(10.)
                            .child(if *count <= 9 {
                                count.to_string()
                            } else {
                                "+".to_string()
                            })
                    }),
            )
    }

    fn render_key(&self) -> DiffKey {
        (&self.server.peek().id).into()
    }
}
