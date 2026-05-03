use std::hash::Hash;

use freya::{icons::lucide::hash, prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{AppChannel, Config};

#[derive(PartialEq)]
pub struct ChannelButton {
    pub channel: Readable<v0::Channel>,
}

impl Component for ChannelButton {
    fn render(&self) -> impl IntoElement {
        let mut config = use_consume::<State<Config>>();
        let mut radio = use_radio(AppChannel::SelectedChannel);
        let selected = radio.slice_current(|state| &state.selected_channel);

        let channel = use_memo({
            let channel = self.channel.clone();
            move || channel.read().clone()
        });

        rect()
            .key(channel.read().id().to_string())
            .horizontal()
            .padding((0., 8., 0., 8.))
            .margin((0., 8., 0., 8.))
            .spacing(8.)
            .height(Size::px(42.))
            .cross_align(Alignment::Center)
            .corner_radius(42.)
            .overflow(Overflow::Clip)
            .font_size(15)
            .child(svg(hash()).width(Size::px(24.)).height(Size::px(24.)))
            .child(channel.read().name().unwrap().to_string())
            .maybe(
                selected.read().as_deref() == Some(channel.read().id()),
                |btn| btn.background(0xff384379).color(0xffdde1ff),
            )
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
    }

    fn render_key(&self) -> DiffKey {
        use std::hash::Hasher;
        let mut hasher = std::hash::DefaultHasher::default();
        self.channel.peek().id().hash(&mut hasher);
        DiffKey::U64(hasher.finish())
    }
}
