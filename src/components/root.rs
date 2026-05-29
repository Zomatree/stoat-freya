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
        let mut config = use_consume::<State<Config>>();

        let token = use_reactive(&config.read().token);

        use_side_effect(move || {
            {
                let token = token.read();
                *http().token.write().unwrap() = token.clone();
            };

            config.write();
        });

        if config.read().token.is_some() && http().token.read().unwrap().is_some() {
            App {}.into_element()
        } else {
            Login {}.into_element()
        }
    }
}
