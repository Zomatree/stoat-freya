use freya::{icons::lucide::x, prelude::*, radio::use_radio};

use crate::{
    AppChannel, Config,
    components::{StoatButton, StoatButtonColorsThemePartialExt, StoatButtonLayoutThemePartialExt},
};

#[derive(PartialEq)]
pub struct Settings {}

impl Component for Settings {
    fn render(&self) -> impl IntoElement {
        let mut config = use_consume::<State<Config>>();
        let radio = use_radio(AppChannel::Settings);

        let close_settings =
            move || radio.clone().write_channel(AppChannel::Settings).settings = None;

        let mut window_size = use_state(Area::default);

        let fullscreen = (window_size.read().width() != 0. && window_size.read().width() <= 1000.)
            || (window_size.read().height() != 0. && window_size.read().height() <= 500.);

        rect()
            .expanded()
            .center()
            .background(0xBB000000)
            .on_press(move |_| close_settings())
            .on_global_key_down(move |e: Event<KeyboardEventData>| {
                if e.key == Key::Named(NamedKey::Escape) {
                    close_settings()
                }
            })
            .on_sized(move |e: Event<SizedEventData>| window_size.set_if_modified(e.area))
            .child(
                rect()
                    .on_press(|e: Event<PressEventData>| e.stop_propagation())
                    .corner_radius(if !fullscreen { 16. } else { 0. })
                    .overflow(Overflow::Clip)
                    .background(0xff34343a)
                    .horizontal()
                    .width(Size::func(move |size| {
                        if !fullscreen {
                            Some((size.parent * 0.8).max(size.parent - 400.).min(size.parent))
                        } else {
                            Some(size.parent)
                        }
                    }))
                    .height(Size::func(move |size| {
                        if !fullscreen {
                            Some((size.parent - 100.).max(300.).min(size.parent))
                        } else {
                            Some(size.parent)
                        }
                    }))
                    .child(
                        rect()
                            .width(Size::px(230.))
                            .height(Size::Fill)
                            .padding(16.)
                            .spacing(16.)
                            .child(
                                StoatButton::new()
                                    .width(Size::fill())
                                    .padding(8.)
                                    .color(Color::RED)
                                    .on_press(move |_| config.write().token = None)
                                    .child("Logout"),
                            ),
                    )
                    .child(
                        rect()
                            .width(Size::Fill)
                            .height(Size::Fill)
                            .corner_radius(CornerRadius {
                                top_left: 16.,
                                top_right: 0.,
                                bottom_right: 0.,
                                bottom_left: 16.,
                                smoothing: 0.,
                            })
                            .background(0xff1b1b21)
                            .horizontal()
                            .child(
                                rect()
                                    .vertical()
                                    .width(Size::func(|size| Some(size.parent - 72.)))
                                    .background(0xff424699),
                            )
                            .child(
                                rect().padding((16., 16.)).child(
                                    StoatButton::new()
                                        .corner_radius(40.)
                                        .background(0xff424659)
                                        .on_press(move |_| close_settings())
                                        .child(
                                            rect()
                                                .center()
                                                .width(Size::px(40.))
                                                .height(Size::px(40.))
                                                .child(
                                                    svg(x())
                                                        .width(Size::px(24.))
                                                        .height(Size::px(24.)),
                                                ),
                                        ),
                                ),
                            ),
                    ),
            )
    }
}
