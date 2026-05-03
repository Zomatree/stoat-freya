use freya::{
    icons::lucide::{
        banknote, chevron_left, circle_plus, compass, house, message_square_text, settings, users,
    },
    prelude::*,
    radio::use_radio,
};

use crate::{AppChannel, Selection, SettingsPage, components::StoatButton, use_config};

#[derive(PartialEq)]
pub struct Welcome {}

impl Component for Welcome {
    fn render(&self) -> impl IntoElement {
        let mut config = use_config();
        let radio = use_radio(AppChannel::Selection);
        let selection = radio.slice_mut_current(|state| &mut state.selection);

        let hide_channel_list = config.read().hide_channel_list;

        rect()
            .child(
                rect()
                    .horizontal()
                    .height(Size::px(48.))
                    .padding((0., 16.))
                    .margin((8., 8., 8., 0.))
                    .spacing(10.)
                    .cross_align(Alignment::Center)
                    .child(
                        StoatButton::new()
                            .on_press(move |_| {
                                config.write().hide_channel_list = !hide_channel_list;
                            })
                            .child(
                        rect()
                            .cross_align(Alignment::Center)
                            .horizontal()
                            .child(
                                svg(chevron_left())
                                    .width(Size::px(20.))
                                    .height(Size::px(20.))
                                    .rotate(if hide_channel_list { 180. } else { 0. }),
                            )
                            .child(svg(house()).width(Size::px(24.)).height(Size::px(24.)))
                        )
                    )
                    .child(label().text("Home").font_size(16))
            )
            .child(
                rect().horizontal().child(
                    rect()
                        .margin((0., 8., 8., 8.))
                        .width(Size::Fill)
                        .height(Size::Fill)
                        .corner_radius(28.)
                        .background(0xff0d0e13)
                        .overflow(Overflow::Clip)
                        .child(
                            rect()
                                .width(Size::Fill)
                                .height(Size::Fill)
                                .padding((48., 8.))
                                .spacing(32.)
                                .center()
                                .child(label().text("Stoat").font_size(46.))
                                .child(
                                    rect()
                                        .spacing(8.)
                                        .padding(8.)
                                        .horizontal()
                                        .child(
                                            rect()
                                                .width(Size::px(260.))
                                                .spacing(8.)
                                                .child(WelcomeButton::new(
                                                    circle_plus(),
                                                    "Create a group or server",
                                                    "Invite all of your friends, some cool bots, and throw a big party.",
                                                    move |_| {}
                                                ))
                                                .child(WelcomeButton::new(
                                                    users(),
                                                    "Go to the Stoat Lounge",
                                                    "You can report issues and discuss improvements with us directly here.",
                                                    move |_| {}
                                                ))
                                                .child(WelcomeButton::new(
                                                    banknote(),
                                                    "Donate to Stoat",
                                                    "Support the project by donating - thank you!",
                                                    move |_| {}
                                                ))
                                        )
                                        .child(
                                            rect()
                                                .width(Size::px(260.))
                                                .spacing(8.)
                                                .child(WelcomeButton::new(
                                                    compass(),
                                                    "Discover Stoat",
                                                    "Find a community based on your hobbies or interests.",
                                                    {
                                                        let mut selection = selection.clone();

                                                        move |_| {
                                                            *selection.write() = Selection::Discover
                                                        }
                                                    }
                                                ))
                                                .child(WelcomeButton::new(
                                                    message_square_text(),
                                                    "Give feedback on Stoat",
                                                    "Let us know how we can improve our app by giving us feedback.",
                                                    move |_| {}
                                                ))
                                                .child(WelcomeButton::new(
                                                    settings(),
                                                    "Open settings",
                                                    "You can also click the gear icon in the bottom left.",
                                                    {
                                                        let mut radio = radio.clone();

                                                        move |_| {
                                                            radio.write_channel(AppChannel::SettingsPage).settings_page = Some(SettingsPage::default())
                                                        }
                                                    }
                                                ))
                                        )
                                ),
                        ),
                ),
            )
    }
}

#[derive(PartialEq)]
struct WelcomeButton {
    icon: Bytes,
    title: &'static str,
    contents: &'static str,
    on_press: EventHandler<Event<PressEventData>>,
}

impl WelcomeButton {
    pub fn new(
        icon: Bytes,
        title: &'static str,
        contents: &'static str,
        on_press: impl Into<EventHandler<Event<PressEventData>>>,
    ) -> Self {
        Self {
            icon,
            title,
            contents,
            on_press: on_press.into(),
        }
    }
}

impl Component for WelcomeButton {
    fn render(&self) -> impl IntoElement {
        let mut hovering = use_state(|| false);

        rect()
            .width(Size::Fill)
            .background(0xff424659)
            .color(0xffdfe1f9)
            .corner_radius(12.)
            .overflow(Overflow::Clip)
            .on_pointer_over(move |_| {
                hovering.set(true);
            })
            .on_pointer_out(move |_| hovering.set_if_modified(false))
            .on_pointer_enter(move |_| {
                Cursor::set(CursorIcon::Pointer);
            })
            .on_pointer_leave(move |_| {
                Cursor::set(CursorIcon::default());
            })
            .child(
                rect()
                    .width(Size::Fill)
                    .padding(13.)
                    .horizontal()
                    .spacing(16.)
                    .cross_align(Alignment::Center)
                    .child(
                        rect()
                            .background(0xff121318)
                            .width(Size::px(36.))
                            .height(Size::px(36.))
                            .corner_radius(36.)
                            .center()
                            .child(
                                svg(self.icon.clone())
                                    .width(Size::px(24.))
                                    .height(Size::px(24.)),
                            ),
                    )
                    .child(
                        rect()
                            .spacing(2.)
                            .child(label().text(self.title).font_size(14))
                            .child(label().text(self.contents).font_size(12)),
                    ),
            )
            .child(
                rect()
                    .position(Position::new_absolute())
                    .width(Size::Fill)
                    .height(Size::Fill)
                    .background(0xffe3e1e9)
                    .opacity(if *hovering.read() { 0.08 } else { 0. }),
            )
            .on_press(self.on_press.clone())
    }
}
