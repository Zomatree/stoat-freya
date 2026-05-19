use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{MessageAttachment, MessageEdit, MessageModel, MessageReactions},
    map_readable,
};

#[derive(PartialEq)]
pub struct MessageContent {
    pub channel: Readable<v0::Channel>,
    pub message: MessageModel,
}

impl Component for MessageContent {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::EditingMessage);

        let editing_message = radio.slice_current(|state| &state.editing_message);

        rect()
            .font_size(14)
            .spacing(4.)
            .maybe_child(
                editing_message
                    .read()
                    .cloned()
                    .filter(|msg| &msg.id == &self.message.message.id)
                    .map(|msg| {
                        MessageEdit {
                            channel: self.channel.clone(),
                            id: msg.id,
                            content: msg.content,
                        }
                        .into_element()
                    })
                    .or_else(|| {
                        self.message
                            .message
                            .content
                            .clone()
                            .filter(|c| !c.is_empty())
                            .map(|content| {
                                SelectableText::new(content)
                                .line_height(1.5)
                                    .into_element()
                                // MarkdownViewer::new(content)
                                //     .paragraph_size(14.)
                                //     .into_element()
                            })
                    }),
            )
            .maybe_child(self.message.message.clone().attachments.and_then(|files| {
                (!files.is_empty()).then(|| {
                    rect().spacing(4.).children(
                        files
                            .into_iter()
                            .map(|file| MessageAttachment { file }.into_element()),
                    )
                })
            }))
            .maybe_child(
                (!self.message.message.reactions.is_empty()).then(|| MessageReactions {
                    message: self.message.clone(),
                    channel: self.channel.clone(),
                }),
            )
    }
}
