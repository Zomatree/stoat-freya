use std::collections::HashMap;

use freya::{
    icons::lucide::{compass, settings},
    prelude::*,
    radio::use_radio,
};
use stoat_models::v0;

use crate::{
    AppChannel, Selection, SettingsPage,
    components::{CurrentUserButton, HomeButton, ServerListButton, StoatButton},
    map_readable,
};

#[derive(PartialEq)]
pub struct ServerList {}

impl Component for ServerList {
    fn render(&self) -> impl IntoElement {
        let mut radio = use_radio(AppChannel::Servers);
        let mut hovered = use_state(|| false);

        let order_settings = radio.slice(AppChannel::Settings("ordering"), |state| {
            &state.settings.ordering
        });

        let servers = radio.slice(AppChannel::Servers, |state| &state.servers);
        let members = radio.slice(AppChannel::Members, |state| &state.members);
        let user_id = radio.slice(AppChannel::UserId, |state| state.user_id.as_ref().unwrap());

        let ordered_servers = use_memo({
            move || {
                let mut order = Vec::new();

                let server_map = servers.read();
                let order_settings = order_settings
                    .read()
                    .as_ref()
                    .and_then(|o| o.servers.as_ref())
                    .cloned();

                if let Some(ordering) = order_settings {
                    for id in ordering.clone() {
                        if server_map.contains_key(&id) {
                            order.push(id);
                        };
                    }

                    for id in server_map.keys() {
                        if !ordering.contains(id) {
                            order.push(id.clone());
                        }
                    }
                } else {
                    let mut join_dates = server_map
                        .keys()
                        .cloned()
                        .map(|id| {
                            let joined_at = members
                                .read()
                                .get(&id)
                                .unwrap()
                                .get(&*user_id.read())
                                .unwrap()
                                .joined_at
                                .clone();

                            (id, joined_at)
                        })
                        .collect::<Vec<_>>();

                    join_dates.sort_by(|(_, a), (_, b)| a.cmp(b));

                    order.extend(join_dates.into_iter().map(|(id, _)| id));
                }

                let readable = servers.clone().into_readable();

                order
                    .into_iter()
                    .map(|id| {
                        map_readable::<HashMap<String, v0::Server>, _>(readable.clone(), move |servers| {
                            servers.get(&id).unwrap()
                        })
                    })
                    .collect::<Vec<_>>()
            }
        });

        rect()
            .child(
                ScrollView::new()
                    .child(
                        rect()
                            .padding((8., 0., 8., 0.))
                            .width(Size::fill())
                            .cross_align(Alignment::Center)
                            .spacing(8.)
                            .child(HomeButton {})
                            .child(CurrentUserButton {})
                            .child(
                                rect()
                                    .height(Size::px(1.))
                                    .width(Size::px(32.))
                                    .background(0xff45464f),
                            )
                            .child(
                                rect()
                                    .width(Size::fill())
                                    .cross_align(Alignment::Center)
                                    .spacing(8.)
                                    .children(
                                        ordered_servers.read().iter().cloned().map(|server| {
                                            ServerListButton { server }.into_element()
                                        }),
                                    ),
                            )
                            .child(
                                StoatButton::new()
                                    .child(
                                        rect()
                                            .center()
                                            .width(Size::px(42.0))
                                            .height(Size::px(42.0))
                                            .corner_radius(42.)
                                            .overflow(Overflow::Clip)
                                            .background(0xff1b1b21)
                                            .color(0xffe3e1e9)
                                            .child(
                                                svg(compass())
                                                    .width(Size::px(32.0))
                                                    .height(Size::px(32.0)),
                                            ),
                                    )
                                    .on_press(move |_| {
                                        radio.write_channel(AppChannel::Selection).selection =
                                            Selection::Discover;
                                    }),
                            ),
                    )
                    .show_scrollbar(false)
                    .width(Size::px(56.))
                    .height(Size::func(|size| Some(size.parent - 56.))),
            )
            .child(
                rect()
                    .width(Size::px(56.))
                    .height(Size::px(56.))
                    .center()
                    .child(
                        StoatButton::new()
                            .child(
                                rect()
                                    .center()
                                    .width(Size::px(42.0))
                                    .height(Size::px(42.0))
                                    .corner_radius(42.)
                                    .overflow(Overflow::Clip)
                                    .background(0xff1b1b21)
                                    .color(0xffe3e1e9)
                                    .child(
                                        svg(settings())
                                            .width(Size::px(32.0))
                                            .height(Size::px(32.0)),
                                    ),
                            )
                            .on_press(move |_| {
                                radio.write_channel(AppChannel::SettingsPage).settings_page =
                                    Some(SettingsPage::default());
                            }),
                    ),
            )
            .on_pointer_enter(move |_| {
                hovered.set(true);
            })
            .on_pointer_leave(move |_| {
                hovered.set(false);
            })
    }
}
