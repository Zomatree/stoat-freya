use std::hash::Hash;

use freya::{icons::lucide::hash, prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{AppChannel, Config, NotificationBadge, get_unread_badge, is_channel_muted};

#[derive(PartialEq)]
pub struct ChannelButton {
    pub channel: Readable<v0::Channel>,
}

impl Component for ChannelButton {
    fn render(&self) -> impl IntoElement {
        let mut config = use_consume::<State<Config>>();
        let mut radio = use_radio(AppChannel::SelectedChannel);
        let selected = radio.slice_current(|state| &state.selected_channel);
        let unreads = radio.slice(AppChannel::ChannelUnreads, |state| &state.channel_unreads);
        let mutes = radio.slice(AppChannel::Settings("notifications"), |state| {
            &state.settings.notifications
        });

        let channel = use_memo({
            let channel = self.channel.clone();
            move || channel.read().clone()
        });

        let unread = use_memo({
            let channel = channel.clone();
            move || unreads.read().get(channel.read().id()).cloned()
        });

        let muted = use_memo({
            let channel = channel.clone();
            let mutes = mutes.into_readable();
            move || is_channel_muted(channel.read().id(), mutes.clone())
        });

        let unread_badge = use_memo({
            let channel = channel.clone();
            let muted = muted.clone();

            move || {
                if *muted.read() {
                    return None;
                };

                let unread = unread.read().clone()?;

                get_unread_badge(&channel.read(), &unread)
            }
        });

        let selected = use_memo(move || selected.read().as_deref() == Some(channel.read().id()));

        rect()
            .key(channel.read().id().to_string())
            .horizontal()
            .content(Content::Flex)
            .padding((0., 8., 0., 8.))
            .margin((0., 8., 0., 8.))
            .spacing(8.)
            .height(Size::px(42.))
            .cross_align(Alignment::Center)
            .corner_radius(42.)
            .overflow(Overflow::Clip)
            .font_size(15)
            .maybe(*selected.read(), |btn| {
                btn.background(0xff384379).color(0xffdde1ff)
            })
            .on_press({
                let channel = channel.clone();

                move |_| {
                    let channel_id = channel.read().id().to_string();

                    radio
                        .write_channel(AppChannel::SelectedChannel)
                        .selected_channel = Some(channel_id.clone());

                    let server_id = match &*channel.read() {
                        v0::Channel::TextChannel { server, .. } => Some(server.clone()),
                        _ => None,
                    };

                    if let Some(server_id) = server_id {
                        config.write().last_channels.insert(server_id, channel_id);
                    };
                }
            })
            .width(Size::Fill)
            .child(svg(hash()).width(Size::px(24.)).height(Size::px(24.)))
            .child(
                label()
                    .text(channel.read().name().unwrap().to_string())
                    .width(Size::flex(1.)),
            )
            .map(
                unread_badge.read().filter(|_| !*selected.read()),
                |this, badge| {
                    this.color(0xffe3e1e9).child(match badge {
                        NotificationBadge::Mentions(count) => rect()
                            .corner_radius(14.)
                            .width(Size::px(14.))
                            .height(Size::px(14.))
                            .center()
                            .background(0xffffb4ab)
                            .color(0xff690005)
                            .font_size(8.)
                            .child(if count <= 9 {
                                count.to_string()
                            } else {
                                "+".to_string()
                            }),
                        NotificationBadge::Unread => rect()
                            .corner_radius(7.)
                            .width(Size::px(7.))
                            .height(Size::px(7.))
                            .background(0xffe3e1e9)
                            .margin((0., 3.5)),
                    })
                },
            )
    }

    fn render_key(&self) -> DiffKey {
        use std::hash::Hasher;
        let mut hasher = std::hash::DefaultHasher::default();
        self.channel.peek().id().hash(&mut hasher);
        DiffKey::U64(hasher.finish())
    }
}
