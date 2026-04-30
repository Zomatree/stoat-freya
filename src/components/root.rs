use freya::prelude::*;

use crate::{
    Config,
    components::{App, Login},
    http,
};

#[derive(PartialEq)]
pub struct Root {}

impl Component for Root {
    fn render(&self) -> impl IntoElement {
        let config = use_consume::<State<Config>>();

        use_side_effect(move || {
            let config = config.read();
            *http().token.write().unwrap() = config.token.clone();
        });

        if config.read().token.is_some() {
            App {}.into_element()
        } else {
            Login {}.into_element()
        }
    }
}
