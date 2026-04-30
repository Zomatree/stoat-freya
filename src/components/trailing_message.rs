use freya::prelude::*;
use stoat_models::v0;

use crate::components::{MessageContent, MessageModel};

#[derive(PartialEq)]
pub struct TrailingMessage {
    pub channel: Readable<v0::Channel>,
    pub message: MessageModel,
}

impl Component for TrailingMessage {
    fn render(&self) -> impl IntoElement {
        rect()
            .horizontal()
            .spacing(8.)
            .child(rect().width(Size::px(54.)).horizontal().center().map(
                self.message.message.read().edited.as_ref(),
                |this, _ts| {
                    this.child(
                        label()
                            .text("(edited)")
                            .font_size(11)
                            .margin((4.5, 0., 0., 0.))
                            .color(Color::DARK_GRAY),
                    )
                },
            ))
            .child(MessageContent {
                channel: self.channel.clone(),
                message: self.message.clone(),
            })
    }
}
