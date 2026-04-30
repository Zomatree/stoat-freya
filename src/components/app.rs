use std::{sync::Arc, time::Duration};

use freya::{
    prelude::*,
    radio::{use_init_radio_station, use_radio, use_radio_station},
};
use tokio::{
    sync::{Mutex, mpsc},
    time::sleep,
};

use crate::{AppChannel, AppState, ConnectionState, components, state, websocket};

#[derive(PartialEq)]
pub struct App {}

impl Component for App {
    fn render(&self) -> impl IntoElement {
        use_init_radio_station::<AppState, AppChannel>(AppState::default);

        let (_msg_s, msg_r) = use_hook(|| {
            let (s, r) = mpsc::unbounded_channel();

            (provide_context(s), Arc::new(Mutex::new(r)))
        });

        let (event_s, event_r) = use_hook(|| {
            let (s, r) = mpsc::unbounded_channel();

            (s, Arc::new(Mutex::new(r)))
        });

        use_future(move || {
            let event_s = event_s.clone();
            let msg_r = msg_r.clone();

            async move {
                loop {
                    if let Err(e) = websocket::run(event_s.clone(), msg_r.clone()).await {
                        println!("{e:?}");
                    };

                    sleep(Duration::from_secs(1)).await;
                }
            }
        });

        let station = use_radio_station::<AppState, AppChannel>();

        use_future(move || {
            let event_r = event_r.clone();
            let station = station.clone();

            async move {
                while let Some(event) = event_r.lock().await.recv().await {
                    state::update_state(event, station);
                    println!("event");
                }
            }
        });

        let radio = use_radio(AppChannel::State);

        match radio.read().state {
            ConnectionState::Connected => components::Client {}.into_element(),
            ConnectionState::Disconnected => rect()
                .width(Size::Fill)
                .height(Size::Fill)
                .center()
                .child(CircularLoader::new())
                .into_element(),
        }
    }
}
