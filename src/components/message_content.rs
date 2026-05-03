use freya::prelude::*;
use stoat_models::v0;

use crate::{
    components::{MessageAttachment, MessageModel, MessageReactions},
    map_readable,
};

#[derive(PartialEq)]
pub struct MessageContent {
    pub channel: Readable<v0::Channel>,
    pub message: MessageModel,
}

impl Component for MessageContent {
    fn render(&self) -> impl IntoElement {
        rect()
            .font_size(14)
            .spacing(4.)
            .maybe_child(
                self.message
                    .message
                    .read()
                    .content
                    .clone()
                    .filter(|c| !c.is_empty())
                    .map(|content| MarkdownViewer::new(content).paragraph_size(14.)),
            )
            .maybe_child(
                (!self.message.message.read().reactions.is_empty()).then(|| MessageReactions {
                    message: self.message.clone(),
                    channel: self.channel.clone(),
                }),
            )
            .maybe_child(
                self.message
                    .message
                    .read()
                    .clone()
                    .attachments
                    .and_then(|files| {
                        (!files.is_empty()).then(|| {
                            rect().spacing(4.).children(files.into_iter().map(|file| {
                                let readable =
                                    map_readable(self.message.message.clone(), move |message| {
                                        message
                                            .attachments
                                            .as_ref()
                                            .unwrap()
                                            .iter()
                                            .find(|f| f.id == file.id)
                                            .unwrap()
                                    });

                                MessageAttachment { file: readable }.into_element()
                            }))
                        })
                    }),
            )
    }
}
