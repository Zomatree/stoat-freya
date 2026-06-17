use std::ops::Not;

use freya::{icons::lucide::settings, prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, ServerSettingsPage,
    components::{Channel, ChannelList, StoatButton, StoatButtonLayoutThemePartialExt, image},
    theme::Theme,
    use_config, use_material_theme,
};

#[derive(PartialEq)]
pub struct Server {
    pub server: Readable<v0::Server>,
}

impl Component for Server {
    fn render(&self) -> impl IntoElement {
        let config = use_config();
        let radio = use_radio(AppChannel::SelectedChannel);
        let theme = use_material_theme();

        let mut show_server_info = use_state(|| false);

        let selected_channel = radio.slice_current(|state| &state.selected_channel);
        let channels = radio.slice(AppChannel::Channels, |state| &state.channels);
        let server_settings = radio.slice_mut(AppChannel::ServerSettingsPage, |state| {
            &mut state.server_settings_page
        });

        let open_server_settings = {
            let server = self.server.clone();
            let server_settings = server_settings.clone();

            move |_| {
                let id = server.read().id.clone();

                *server_settings.clone().write() = Some((id, ServerSettingsPage::default()));
            }
        };

        let server_header = rect()
            .horizontal()
            .content(Content::Flex)
            .cross_align(Alignment::Center)
            .spacing(8.)
            .child(
                label()
                    .font_size(16)
                    .text(self.server.read().name.clone())
                    .width(Size::flex(1.)),
            )
            .child(
                StoatButton::new()
                    .corner_radius(16.)
                    .on_press(open_server_settings.clone())
                    .child(
                        rect()
                            .padding(4.)
                            .child(svg(settings()).width(Size::px(24.)).height(Size::px(24.))),
                    ),
            )
            .into_element();

        rect()
            .corner_radius(CornerRadius {
                top_left: 16.,
                top_right: 0.,
                bottom_right: 0.,
                bottom_left: 16.,
                smoothing: 0.,
            })
            .background(theme.md.surface_container_low.as_argb_u32())
            .overflow(Overflow::Clip)
            .direction(Direction::Horizontal)
            .maybe_child(config.read().hide_channel_list.not().then(|| {
                rect()
                    .spacing(8.)
                    .child(
                        rect()
                            .margin((8., 8., 0., 8.))
                            .on_press(move |_| show_server_info.set(true))
                            .on_pointer_enter(move |_| {
                                Cursor::set(CursorIcon::Pointer);
                            })
                            .on_pointer_leave(move |_| {
                                Cursor::set(CursorIcon::default());
                            })
                            .child(if let Some(banner) = &self.server.read().banner {
                                rect()
                                    .height(Size::px(120.))
                                    .width(Size::px(240.))
                                    .corner_radius(16.)
                                    .overflow(Overflow::Clip)
                                    .child(image(&banner).aspect_ratio(AspectRatio::Max))
                                    .child(
                                        rect()
                                            .width(Size::Fill)
                                            .position(Position::new_absolute().bottom(0.))
                                            .layer(Layer::Relative(1))
                                            .padding((6., 14.))
                                            .background_linear_gradient(
                                                LinearGradient::new()
                                                    .stop((Color::TRANSPARENT, 0.))
                                                    .stop((Color::BLACK, 90.)),
                                            )
                                            .corner_radius(CornerRadius {
                                                top_left: 0.,
                                                top_right: 0.,
                                                bottom_right: 16.,
                                                bottom_left: 16.,
                                                smoothing: 0.,
                                            })
                                            .overflow(Overflow::Clip)
                                            .child(server_header.clone()),
                                    )
                            } else {
                                rect()
                                    .padding((0., 16.))
                                    .height(Size::px(48.))
                                    .center()
                                    .child(server_header)
                            }),
                    )
                    .child(ChannelList {
                        server: self.server.clone(),
                    })
                    .width(Size::px(248.))
            }))
            .child(
                if let Some(channel) = selected_channel.read().clone().and_then(|channel| {
                    if channels.read().contains_key(&channel) {
                        Some(radio.slice(AppChannel::Channels, move |state| {
                            state.channels.get(&channel).unwrap()
                        }))
                    } else {
                        None
                    }
                }) {
                    Channel {
                        channel: channel.into_readable(),
                        server: Some(self.server.clone()),
                    }
                    .into_element()
                } else {
                    "No selected channel".into_element()
                },
            )
            .child(
                Popup::new()
                    .background(theme.md.surface_container_high.as_argb_u32())
                    .color(theme.md.on_surface.as_argb_u32())
                    .padding(24.)
                    .width(Size::px(560.))
                    .on_close_request(move |_| show_server_info.set(false))
                    .maybe(show_server_info(), move |popup| {
                        let server = self.server.read();

                        popup
                            .child(
                                label()
                                    .font_size(24.)
                                    .line_height(1.5)
                                    .text(server.name.clone())
                                    .margin((0., 0., 16., 0.)),
                            )
                            .maybe_child(server.description.clone().map(|description| {
                                label()
                                    .color(theme.md.on_surface_variant.as_argb_u32())
                                    .font_weight(400)
                                    .font_size(14.)
                                    .text(description)
                            }))
                            .child(
                                rect()
                                    .font_size(14.)
                                    .width(Size::Fill)
                                    .margin((24., 0., 0., 0.))
                                    .horizontal()
                                    .main_align(Alignment::End)
                                    .child(popup_button(
                                        &theme,
                                        show_server_info,
                                        "Settings",
                                        open_server_settings,
                                    ))
                                    .child(popup_button(
                                        &theme,
                                        show_server_info,
                                        "Close",
                                        move |_| {},
                                    )),
                            )
                    }),
            )
    }

    fn render_key(&self) -> DiffKey {
        (&self.server.peek().id).into()
    }
}

fn popup_button(
    theme: &Theme,
    mut show_server_info: State<bool>,
    title: &'static str,
    callback: impl Into<EventHandler<Event<PressEventData>>>,
) -> StoatButton {
    let callback = callback.into();

    StoatButton::new()
        .corner_radius(20.)
        .on_press(move |e| {
            show_server_info.set(false);
            callback.call(e)
        })
        .child(
            rect()
                .padding((0., 16.))
                .height(Size::px(40.))
                .main_align(Alignment::Center)
                .color(theme.md.primary.as_argb_u32())
                .child(title),
        )
}
