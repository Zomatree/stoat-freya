use std::collections::HashMap;

use freya::{
    icons::lucide::{compass, plus, settings},
    prelude::*,
    radio::use_radio,
};
use stoat_models::v0;

use crate::{
    AppChannel, Selection, SettingsPage,
    components::{
        CurrentUserButton, HomeButton, ModalValue, ServerListButton, StoatButton, StoatButtonLayoutThemePartialExt, StoatTooltip, use_modals
    },
    map_readable, use_material_theme,
};

#[derive(PartialEq)]
pub struct ServerList {}

impl Component for ServerList {
    fn render(&self) -> impl IntoElement {
        let mut radio = use_radio(AppChannel::Servers);
        let theme = use_material_theme();
        let mut modals = use_modals();

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
                        map_readable::<HashMap<String, v0::Server>, _>(
                            readable.clone(),
                            move |servers| servers.get(&id).unwrap(),
                        )
                    })
                    .collect::<Vec<_>>()
            }
        });

        rect()
            .child(
                ScrollView::new()
                    .child(
                        rect()
                            // .padding((8., 0.))
                            .width(Size::fill())
                            .cross_align(Alignment::Center)
                            // .spacing(8.)
                            .child(HomeButton {})
                            .child(CurrentUserButton {})
                            .child(
                                rect()
                                    .height(Size::px(1.))
                                    .width(Size::px(32.))
                                    .margin((6., 0.))
                                    .background(theme.md.outline_variant.as_argb_u32()),
                            )
                            .child(
                                rect()
                                    .width(Size::fill())
                                    .cross_align(Alignment::Center)
                                    // .spacing(8.)
                                    .children(
                                        ordered_servers.read().iter().cloned().map(|server| {
                                            ServerListButton { server }.into_element()
                                        }),
                                    ),
                            )
                            .child(
                                StoatTooltip::new(
                                    label().max_lines(1).font_size(11.).text("Create or join a server"),
                                )
                                .position(AttachedPosition::Right)
                                .child(
                                    rect()
                                        .width(Size::px(56.))
                                        .height(Size::px(56.))
                                        .center()
                                        .child(
                                            StoatButton::new()
                                                .corner_radius(42.)
                                                .child(
                                                    rect()
                                                        .center()
                                                        .width(Size::px(42.0))
                                                        .height(Size::px(42.0))
                                                        .background(
                                                            theme
                                                                .md
                                                                .surface_container_low
                                                                .as_argb_u32(),
                                                        )
                                                        .child(
                                                            svg(plus())
                                                                .width(Size::px(32.0))
                                                                .height(Size::px(32.0)),
                                                        ),
                                                )
                                                .on_press(move |_| {
                                                    modals.write().push_modal(ModalValue::CreateJoinServer);
                                                }),
                                        ),
                                ),
                            )
                            .child(
                                StoatTooltip::new(
                                    label().max_lines(1).font_size(11.).text("Find new servers to join"),
                                )
                                .position(AttachedPosition::Right)
                                .child(
                                    rect()
                                        .width(Size::px(56.))
                                        .height(Size::px(56.))
                                        .center()
                                        .child(
                                            StoatButton::new()
                                                .corner_radius(42.)
                                                .child(
                                                    rect()
                                                        .center()
                                                        .width(Size::px(42.0))
                                                        .height(Size::px(42.0))
                                                        .background(
                                                            theme
                                                                .md
                                                                .surface_container_low
                                                                .as_argb_u32(),
                                                        )
                                                        .child(
                                                            svg(compass())
                                                                .width(Size::px(32.0))
                                                                .height(Size::px(32.0)),
                                                        ),
                                                )
                                                .on_press(move |_| {
                                                    radio
                                                        .write_channel(AppChannel::Selection)
                                                        .selection = Selection::Discover;
                                                }),
                                        ),
                                ),
                            )
                    )
                    .show_scrollbar(false)
                    .width(Size::px(56.))
                    .height(Size::func(|size| Some(size.parent - 56.))),
            )
            .child(
                StoatTooltip::new(label().max_lines(1).font_size(11.).text("Settings")).position(AttachedPosition::Right).child(
                    rect()
                        .width(Size::px(56.))
                        .height(Size::px(56.))
                        .center()
                        .child(
                            StoatButton::new()
                                .corner_radius(42.)
                                .child(
                                    rect()
                                        .center()
                                        .width(Size::px(42.0))
                                        .height(Size::px(42.0))
                                        .overflow(Overflow::Clip)
                                        .background(theme.md.surface_container_low.as_argb_u32())
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
                ),
            )
    }
}
