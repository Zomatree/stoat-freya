use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{Category, ChannelButton},
    map_readable,
};

#[derive(PartialEq)]
pub struct ChannelList {
    pub server: Readable<v0::Server>,
}

impl Component for ChannelList {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Channels);

        let categories = self.server.read().categories.clone().unwrap_or_default();

        let category_channels = categories
            .iter()
            .flat_map(|cat| cat.channels.clone())
            .collect::<Vec<_>>();

        let non_category_channels = self
            .server
            .read()
            .channels
            .iter()
            .filter(|&channel_id| !category_channels.contains(channel_id))
            .cloned()
            .collect::<Vec<_>>();

        ScrollView::new().child(
            rect()
                .padding(8.)
                .cross_align(Alignment::Center)
                .color(0xff90909a)
                .children(
                    non_category_channels
                        .into_iter()
                        .filter(|channel_id| radio.read().channels.contains_key(channel_id))
                        .map(|channel_id: String| {
                            let channel = radio.slice_current(move |state| {
                                state.channels.get(&channel_id).unwrap()
                            });

                            ChannelButton {
                                channel: channel.into_readable(),
                            }
                            .into_element()
                        }),
                )
                .children(
                    categories
                        .into_iter()
                        .filter(|cat| !cat.channels.is_empty())
                        .map(|cat| {
                            let server = self.server.clone();

                            let category = map_readable(server, {
                                let id = cat.id.clone();

                                move |server| {
                                    server
                                        .categories
                                        .as_ref()
                                        .unwrap()
                                        .iter()
                                        .find(|c| c.id == id)
                                        .unwrap()
                                }
                            });

                            // let channels = cat
                            //     .channels
                            //     .into_iter()
                            //     .filter(|channel_id| radio.read().channels.contains_key(channel_id))
                            //     .map(|channel_id: String| {
                            //         radio
                            //             .slice_current(move |state| {
                            //                 state.channels.get(&channel_id).unwrap()
                            //             })
                            //             .into_readable()
                            //     })
                            //     .collect::<Vec<Readable<v0::Channel>>>();

                            // println!("chanlist: {:?}", channels.iter().map(|c| c.peek().name().map(|s| s.to_string())).collect::<Vec<_>>());

                            category
                        })
                        .map(|category| {
                            rect()
                                .key(category.peek().id.clone())
                                .child(Category {
                                    category: category,
                                })
                                .into_element()
                        }),
                )
                .width(Size::Fill),
        )
    }
}
