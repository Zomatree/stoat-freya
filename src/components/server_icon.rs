use freya::prelude::*;
use stoat_models::v0::Server;

use crate::components::image;

pub fn server_icon(server: &Server) -> impl IntoElement {
    rect().child(match &server.icon {
        Some(file) => image(file).into_element(),
        None => rect()
            .background(0xff1b1b21)
            .width(Size::Fill)
            .height(Size::Fill)
            .center()
            .child(&server.name[0..1])
            .color(0xffe3e1e9)
            .into_element(),
    })
}
