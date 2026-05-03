use std::time::Duration;

use freya::{prelude::*, radio::use_radio};
use tokio::time::sleep;

use crate::{
    AppChannel, ConnectionState, Selection,
    components::{Discover, Home, Server, ServerList, Settings, UserProfile},
};

#[derive(PartialEq)]
pub struct Client {}

impl Component for Client {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Selection);
        let selected = radio.slice_current(|state| &state.selection);
        let settings = radio.slice(AppChannel::SettingsPage, |state| &state.settings_page);
        let connection_state = radio.slice_mut(AppChannel::State, |state| &mut state.state);
        let user_profile = radio.slice_mut(AppChannel::UserProfile, |state| &mut state.user_profile);

        let show_connection_state_banner = use_state(|| false);

        use_side_effect({
            let connection_state = connection_state.clone();

            move || {
                let mut show_connection_state_banner = show_connection_state_banner.clone();

                match *connection_state.read() {
                    ConnectionState::Disconnected => *show_connection_state_banner.write() = true,
                    ConnectionState::Reconnecting => *show_connection_state_banner.write() = true,
                    ConnectionState::Reconnected => {
                        *show_connection_state_banner.write() = true;

                        spawn({
                            let mut connection_state = connection_state.clone();
                            async move {
                                sleep(Duration::from_secs_f32(2.5)).await;

                                *show_connection_state_banner.write() = false;
                                *connection_state.write() = ConnectionState::Connected;
                            }
                        });
                    }
                    ConnectionState::Connected => {},
                }
            }
        });

        rect()
            .color(0xffe3e1e9)
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .child(ServerList {})
                    .child(match selected.read().clone() {
                        Selection::Server(server_id) => {
                            let server = radio.slice(AppChannel::Servers, move |state| {
                                state.servers.get(&server_id).unwrap()
                            });

                            Server {
                                server: server.into_readable(),
                            }
                            .into_element()
                        }
                        Selection::Discover => Discover {}.into_element(),
                        Selection::Home => Home {}.into_element(),
                    }),
            )
            .maybe_child(settings.read().is_some().then(|| {
                rect()
                    .position(Position::new_global())
                    .width(Size::window_percent(100.))
                    .height(Size::window_percent(100.))
                    .layer(Layer::Overlay)
                    .child(Settings {})
                    .into_element()
            }))
            .maybe_child(show_connection_state_banner.read().then(|| {
                let connection_state = connection_state.read();

                let (color, background, text) = match *connection_state {
                    ConnectionState::Disconnected => (0xff90909a, 0xff292a2f, "Disconnected"),
                    ConnectionState::Connected => (0xffdde1ff, 0xff384379, "Connected"),
                    ConnectionState::Reconnecting => (0xff90909a, 0xff292a2f, "Reconnecting"),
                    ConnectionState::Reconnected => (0xffdde1ff, 0xff384379, "Reconnected"),
                };

                rect()
                    .position(Position::new_global())
                    .layer(Layer::Overlay)
                    .width(Size::window_percent(100.))
                    .height(Size::px(30.))
                    .center()
                    .child(label().text(text).font_size(16.).color(color).font_weight(FontWeight::SEMI_BOLD))
                    .background(background)
            }))
            .maybe_child(user_profile.read().cloned().map(|user_id| {
                let user = radio.slice(AppChannel::Users, move |state| state.users.get(&user_id).unwrap());

                rect()
                    .position(Position::new_global())
                    .width(Size::window_percent(100.))
                    .height(Size::window_percent(100.))
                    .layer(Layer::Overlay)
                    .child(UserProfile { user: user.into_readable() })
                    .into_element()
            }))
    }
}
