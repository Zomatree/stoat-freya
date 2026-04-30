use freya::{prelude::*, radio::use_radio};

use crate::{
    AppChannel, Selection,
    components::{Discover, Home, Server, ServerList, Settings},
};

#[derive(PartialEq)]
pub struct Client {}

impl Component for Client {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Selection);
        let selected = radio.slice_current(|state| &state.selection);
        let settings = radio.slice(AppChannel::Settings, |state| &state.settings);

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
    }
}
