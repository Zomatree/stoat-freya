use std::time::Duration;

use freya::{
    animation::{AnimNum, Ease, OnChange, OnCreation, use_animation},
    icons::lucide::chevron_down,
    prelude::*,
    radio::use_radio,
};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{ChannelButton, StoatButton, StoatButtonLayoutThemePartialExt},
    use_config,
};

#[derive(PartialEq)]
pub struct Category {
    pub category: Readable<v0::Category>,
}

impl Component for Category {
    fn render(&self) -> impl IntoElement {
        let mut config = use_config();
        let radio = use_radio(AppChannel::Channels);
        let selected_channel =
            radio.slice(AppChannel::SelectedChannel, |state| &state.selected_channel);

        // let category = use_memo({
        //     let category = self.category.clone();
        //     move || category.read().clone()
        // });

        let channels = use_side_effect_value({
            let category = self.category.clone();

            move || {
                category
                    .read()
                    .channels
                    .iter()
                    .filter(|&channel_id| radio.read().channels.contains_key(channel_id))
                    .cloned()
                    .map(|channel_id| {
                        radio
                            .slice(AppChannel::Channels, move |state| {
                                state.channels.get(&channel_id).unwrap()
                            })
                            .into_readable()
                    })
                    .collect::<Vec<Readable<v0::Channel>>>()
            }
        });

        let is_expanded = use_memo({
            let category = self.category.clone();
            move || {
                !config
                    .read()
                    .collapsed_categories
                    .contains(&category.read().id)
            }
        });

        let animation = use_animation(move |conf| {
            conf.on_change(OnChange::Rerun);
            conf.on_creation(OnCreation::Finish);

            let (start, end) = if *is_expanded.read() {
                (-90., 0.)
            } else {
                (0., -90.)
            };

            AnimNum::new(start, end)
                .ease(Ease::InOut)
                .duration(Duration::from_millis(100))
        });

        rect()
            .key(self.category.read().id.clone())
            .spacing(8.)
            .child(
                StoatButton::new()
                    .width(Size::Fill)
                    .child(
                        rect()
                            .padding((10., 0., 0., 12.))
                            .color(0xffe3e1e9)
                            .horizontal()
                            .cross_align(Alignment::Center)
                            .main_align(Alignment::Start)
                            .spacing(4.)
                            .child(
                                label()
                                    .font_size(13)
                                    .max_lines(1)
                                    .text_overflow(TextOverflow::Ellipsis)
                                    .text(self.category.read().title.clone()),
                            )
                            .child(
                                svg(chevron_down())
                                    .width(Size::px(12.))
                                    .height(Size::px(12.))
                                    .rotate(&*animation.read()),
                            ),
                    )
                    .on_press({
                        let category_id = self.category.read().id.clone();

                        move |_| {
                            let collapsed = &mut config.write().collapsed_categories;

                            if *is_expanded.read() {
                                collapsed.insert(category_id.clone());
                            } else {
                                collapsed.remove(&category_id);
                            };
                        }
                    }),
            )
            .maybe_child(
                is_expanded
                    .read()
                    .then(|| {
                        rect().children(channels.read().iter().map(|channel| {
                            rect()
                                .key(channel.peek().id())
                                .child(ChannelButton {
                                    channel: channel.clone(),
                                })
                                .into_element()
                        }))
                    })
                    .or({
                        let selected = selected_channel.read();

                        if let Some(id) = &*selected
                            && let Some(channel) = channels
                                .read()
                                .iter()
                                .find(|channel| channel.peek().id() == id)
                        {
                            Some(rect().key(channel.peek().id()).child(ChannelButton {
                                channel: channel.clone(),
                            }))
                        } else {
                            None
                        }
                    }),
            )
    }

    fn render_key(&self) -> DiffKey {
        (&self.category.peek().id).into()
    }
}
