use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{AppChannel, components::Avatar};

#[derive(PartialEq)]
pub struct UserProfile {
    pub user: Readable<v0::User>,
}

impl Component for UserProfile {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::UserProfile);

        let close_profile = move || radio.clone().write().user_profile = None;

        let user = use_memo({
            let user = self.user.clone();
            move || user.read().clone()
        });

        rect()
            .expanded()
            .center()
            .background(0xBB000000)
            .on_press(move |_| close_profile())
            .on_global_key_down(move |e: Event<KeyboardEventData>| {
                if e.key == Key::Named(NamedKey::Escape) {
                    close_profile()
                }
            })
            .child(
                rect()
                    .on_press(|e: Event<PressEventData>| e.stop_propagation())
                    .corner_radius(28.)
                    .overflow(Overflow::Clip)
                    .background(0xff292a2f)
                    .width(Size::px(560.))
                    .padding(8.)
                    .child(
                        rect().padding(8.).child(
                            rect()
                                .horizontal()
                                .child(Avatar::new(self.user.clone(), None, 48.).presence(true))
                                .child(format!(
                                    "{}#{}",
                                    user.read().username,
                                    user.read().discriminator
                                )),
                        ),
                    ),
            )
    }
}
