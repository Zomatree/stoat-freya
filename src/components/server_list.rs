use freya::{
    icons::lucide::{compass, settings},
    prelude::*,
    radio::use_radio,
};

use crate::{
    AppChannel, Config, Selection, SettingsPage,
    components::{CurrentUserButton, HomeButton, StoatButton, server_icon},};

#[derive(PartialEq)]
pub struct ServerList {}

impl Component for ServerList {
    fn render(&self) -> impl IntoElement {
        let config = use_consume::<State<Config>>();
        let mut radio = use_radio(AppChannel::Servers);
        let mut hovered = use_state(|| false);

        let selection = radio.slice(AppChannel::Selection, |state| &state.selection);
        let order_settings = radio.slice(AppChannel::Settings("ordering"), |state| {
            &state.settings.ordering
        });
        let servers = radio.slice(AppChannel::Servers, |state| &state.servers);
        let members = radio.slice(AppChannel::Members, |state| &state.members);
        let user_id = radio.slice(AppChannel::UserId, |state| state.user_id.as_ref().unwrap());

        let ordered_servers = use_memo({
            move || {
                let mut order = Vec::new();

                let servers = servers.read();
                let order_settings = order_settings.read().as_ref().and_then(|o| o.servers.as_ref()).cloned();

                if let Some(ordering) = order_settings {
                    for id in ordering.clone() {
                        if servers.contains_key(&id) {
                            order.push(id);
                        };
                    }

                    for id in servers.keys() {
                        if !ordering.contains(id) {
                            order.push(id.clone());
                        }
                    }
                } else {
                    let mut join_dates = servers
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

                order
                    .iter()
                    .map(|id| servers.get(id).unwrap().clone())
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
                            .children(ordered_servers.read().iter().cloned().map(|server| {
                                rect()
                                    .key(server.id.clone())
                                    .maybe_child(
                                        (&*selection.read()
                                            == &Selection::Server(server.id.clone()))
                                            .then(|| {
                                                rect()
                                                    .width(Size::px(12.))
                                                    .height(Size::px(32.))
                                                    .layer(Layer::RelativeOverlay(1))
                                                    .position(
                                                        Position::new_absolute().left(-16.).top(5.),
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
                                            .child(server_icon(&server))
                                            .on_press({
                                                move |_| {
                                                    radio
                                                        .write_channel(AppChannel::Selection)
                                                        .selection =
                                                        Selection::Server(server.id.clone());

                                                    let channel_id = config
                                                        .read()
                                                        .last_channels
                                                        .get(&server.id)
                                                        .or_else(|| {
                                                            let channels = radio.slice(
                                                                AppChannel::Channels,
                                                                |state| &state.channels,
                                                            );

                                                            server.channels.iter().find(|&id| {
                                                                channels.read().contains_key(id)
                                                            })
                                                        })
                                                        .cloned();

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
                                    .into_element()
                            }))
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
