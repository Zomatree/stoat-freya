use freya::{icons::lucide::file_text, prelude::*};
use stoat_models::v0;

use crate::components::{MessageModel, avatar::avatar};

#[derive(PartialEq)]
pub struct MessageReply {
    pub channel: Readable<v0::Channel>,
    pub message: MessageModel,
    pub reply: MessageModel,
}

impl Component for MessageReply {
    fn render(&self) -> impl IntoElement {
        let has_attachments = self
            .reply
            .message
            .read()
            .attachments
            .as_ref()
            .is_some_and(|files| !files.is_empty());

        rect()
            .height(Size::px(22.))
            .horizontal()
            .font_size(14)
            .spacing(4.)
            .cross_align(Alignment::Center)
            .child(
                rect()
                    .height(Size::px(12.))
                    .width(Size::px(22.))
                    .margin((0., 6., 0., 30.))
                    .corner_radius(4.)
                    .border(Border::new().fill(0xff45464f).width(BorderWidth {
                        top: 2.,
                        right: 0.,
                        bottom: 0.,
                        left: 2.,
                    })),
            )
            .child(
                rect()
                    .height(Size::px(22.))
                    .horizontal()
                    .spacing(4.)
                    .cross_align(Alignment::Center)
                    .child(
                        self.reply
                            .member
                            .as_ref()
                            .map(|r| avatar(&self.reply.user.read(), Some(&r.read())))
                            .unwrap_or_else(|| avatar(&self.reply.user.read(), None))
                            .width(Size::px(14.))
                            .height(Size::px(14.)),
                    )
                    .child(
                        label()
                            .text({
                                let user = self.reply.user.read();

                                if self
                                    .message
                                    .message
                                    .read()
                                    .mentions
                                    .as_ref()
                                    .is_some_and(|mentions| mentions.contains(&user.id))
                                {
                                    format!("@{}", user.username)
                                } else {
                                    user.username.clone()
                                }
                            })
                            .line_height(1.5),
                    ),
            )
            .maybe_child(
                has_attachments
                    .then(|| svg(file_text()).width(Size::px(16.)).height(Size::px(16.))),
            )
            .child(
                if let Some(content) = self.reply.message.read().content.clone()
                    && !content.is_empty()
                {
                    label()
                        .text(content)
                        .max_lines(1)
                        .line_height(1.5)
                        .text_overflow(TextOverflow::Ellipsis)
                        .into_element()
                } else if has_attachments {
                    label()
                        .line_height(1.5)
                        .font_slant(FontSlant::Italic)
                        .text("Sent an attachment")
                        .into_element()
                } else {
                    rect().into_element()
                },
            )
    }
}
