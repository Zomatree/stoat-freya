use freya::{prelude::*, webview::WebViewPlugin};
use tokio::runtime::Builder;

pub mod components;
pub mod config;
pub mod error;
pub mod http;
pub mod state;
pub mod types;
pub mod utils;
pub mod websocket;

pub use config::*;
pub use error::*;
pub use http::*;
pub use state::*;
pub use utils::*;

use crate::components::{StoatButtonColorsThemePreference, StoatButtonLayoutThemePreference};

pub const BASE: &str = "https://api.stoat.chat";

fn app() -> impl IntoElement {
    let config = use_hook(|| {
        let state = State::create(read_config());
        provide_context(state);

        state
    });

    use_side_effect(move || {
        let new_value = config.read();

        write_config(&new_value);
    });

    use_init_theme(|| {
        let mut theme = dark_theme();

        theme.set(
            "stoat_button",
            StoatButtonColorsThemePreference {
                background: Preference::Specific(Color::TRANSPARENT),
                hover_background: Preference::Specific(Color::TRANSPARENT),
                border_fill: Preference::Specific(Color::TRANSPARENT),
                focus_border_fill: Preference::Specific(Color::TRANSPARENT),
                color: Preference::Reference("text_primary"),
            },
        );

        theme.set(
            "stoat_button_layout",
            StoatButtonLayoutThemePreference {
                margin: Preference::Specific(Gaps::new_all(0.)),
                corner_radius: Preference::Specific(CornerRadius::new_all(0.)),
                width: Preference::Specific(Size::Inner),
                height: Preference::Specific(Size::Inner),
                padding: Preference::Specific(Gaps::new_all(0.)),
            },
        );

        theme
    });

    let future = use_future(move || async {
        let http = http::HttpClient::new(BASE.to_string(), None).await.unwrap();
        HTTP.set(http).unwrap();
    });

    rect()
        .font_family("Inter")
        .background(0xff292a2f)
        .color(0xffe3e1e9)
        .width(Size::Fill)
        .height(Size::Fill)
        .child(match *future.state() {
            FutureState::Fulfilled(_) => components::Root {}.into_element(),
            _ => rect()
                .width(Size::Fill)
                .height(Size::Fill)
                .center()
                .child(CircularLoader::new())
                .into_element(),
        })
}

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "info");
        }
    };

    pretty_env_logger::init();

    let rt = Builder::new_multi_thread().enable_all().build().unwrap();
    let _rt = rt.enter();

    launch(
        LaunchConfig::new()
            .with_window(
                WindowConfig::new(app)
                    .with_title("Stoat")
                    .with_app_id("chat.stoat.app")
                    .with_size(1280., 720.),
            )
            .with_plugin(WebViewPlugin::new()),
    );
}
