use freya::{icons::lucide::house, prelude::*, radio::use_radio};

use crate::{
    AppChannel, Selection,
    components::{StoatButton, StoatButtonColorsThemePartialExt, StoatButtonLayoutThemePartialExt},
};

#[derive(PartialEq)]
pub struct HomeButton {}

impl Component for HomeButton {
    fn render(&self) -> impl IntoElement {
        let mut radio = use_radio(AppChannel::Selection);

        StoatButton::new()
            .width(Size::px(56.))
            .height(Size::px(56.))
            .child(
                rect().expanded().center().child(
                    rect()
                        .background(0xff1b1b21)
                        .width(Size::px(42.))
                        .height(Size::px(42.))
                        .corner_radius(42.)
                        .center()
                        .child(
                            svg(house())
                                .width(Size::px(24.))
                                .height(Size::px(24.))
                                .color(0xffe3e1e9),
                        ),
                ),
            )
            .on_press(move |_| {
                radio.write().selection = Selection::Home;
                radio
                    .write_channel(AppChannel::SelectedChannel)
                    .selected_channel = None;
            })

        // rect()
        //     .corner_radius(42.)
        //     .overflow(Overflow::Clip)
        //     .width(Size::px(42.))
        //     .height(Size::px(42.))
        //     .center()
        //     .child(
        //         svg(house())
        //             .width(Size::px(24.))
        //             .height(Size::px(24.))
        //             .color(0xffe3e1e9),
        //     )
        //     .background(0xff1b1b21)
        //     .on_pointer_enter(|_| {
        //         Cursor::set(CursorIcon::Pointer);
        //     })
        //     .on_pointer_leave(|_| {
        //         Cursor::set(CursorIcon::Default);
        //     })
        //     .on_press(move |_| {
        //         radio.write().selection = Selection::Home;
        //         radio
        //             .write_channel(AppChannel::SelectedChannel)
        //             .selected_channel = None;
        //     })
    }
}
