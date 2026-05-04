use freya::{icons::lucide::ellipsis_vertical, prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{AppChannel, components::{Avatar, StoatButton}, http};

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

        let profile = use_state(|| None);

        use_future({
            let user_id = self.user.peek().id.clone();

            move || {
                let user_id = user_id.clone();
                let mut profile = profile.clone();

                async move {
                    if let Ok(user_profile) = http().fetch_user_profile(&user_id).await {
                        profile.set(Some(user_profile));
                    }
                }
            }
        });

        let profile = profile.read().clone();

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
                        rect()
                            .padding(8.)
                            .spacing(8.)
                            .child(
                                rect()
                                    .main_align(Alignment::End)
                                    .width(Size::Fill)
                                    .padding(15.)
                                    .height(Size::px(120.))
                                    .corner_radius(28.)
                                    .background_linear_gradient(
                                        LinearGradient::new()
                                            .stop((0x33000000, 20.))
                                            .stop((0xb3000000, 70.)),
                                    )
                                    .child(
                                        rect()
                                            .horizontal()
                                            .spacing(15.)
                                            .cross_align(Alignment::Center)
                                            .child(
                                                Avatar::new(self.user.clone(), None, 48.)
                                                    .presence(true),
                                            )
                                            .child(format!(
                                                "{}#{}",
                                                user.read().username,
                                                user.read().discriminator
                                            )),
                                    ),
                            )
                            .child(
                                rect().horizontal().width(Size::Fill).main_align(Alignment::End).child(
                                    StoatButton::new().on_press(move |_| {}).child(
                                        rect()
                                            .horizontal()
                                            .height(Size::px(40.))
                                            .padding((0., 8.))
                                            .center()
                                            .child(
                                                svg(ellipsis_vertical())
                                                    .width(Size::px(24.))
                                                    .height(Size::px(24.)),
                                            ),
                                    ),
                                ),
                            )
                            .maybe_child(profile.as_ref().and_then(|profile| {
                                profile.content.as_ref().map(|content| {
                                    rect()
                                        .width(Size::Fill)
                                        .padding(15.)
                                        .background(0xff1b1b21)
                                        .corner_radius(16.)
                                        .spacing(4.)
                                        .child(
                                            label()
                                                .text("Bio")
                                                .font_size(22.)
                                                .font_weight(FontWeight::SEMI_BOLD),
                                        )
                                        .child(label().text(content.clone()).font_size(14))
                                })
                            })),
                    ),
            )
    }
}
