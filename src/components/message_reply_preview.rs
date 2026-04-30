use freya::{icons::lucide::file_text, prelude::*};
use stoat_models::v0;

use crate::components::{ReplyController, ReplyIntent, avatar::avatar};

#[derive(PartialEq)]
pub struct MessageReplyPreview {
    pub replies: ReplyController,
    pub reply: Readable<ReplyIntent>,
    pub channel: Readable<v0::Channel>,
}

impl Component for MessageReplyPreview {
    fn render(&self) -> impl IntoElement {
        let message = self.reply.read().message.clone();

        let has_attachments = message
            .message
            .read()
            .attachments
            .as_ref()
            .is_some_and(|files| !files.is_empty());

        rect()
            .background(0xff384379)
            .corner_radius(16.)
            .overflow(Overflow::Clip)
            .width(Size::func(|size| Some(size.parent - 16.)))
            .padding((8., 16., 8., 16.))
            .horizontal()
            .font_size(14)
            .spacing(4.)
            .cross_align(Alignment::Center)
            .child(label().font_size(12).text("Replying to"))
            .child(
                rect()
                    .horizontal()
                    .spacing(4.)
                    .cross_align(Alignment::Center)
                    .child(
                        message
                            .member
                            .as_ref()
                            .map(|r| avatar(&message.user.read(), Some(&r.read())))
                            .unwrap_or_else(|| avatar(&message.user.read(), None))
                            .width(Size::px(14.))
                            .height(Size::px(14.)),
                    )
                    .child(message.user.read().username.clone())
                    .maybe_child(
                        has_attachments
                            .then(|| svg(file_text()).width(Size::px(16.)).height(Size::px(16.))),
                    )
                    .child(
                        if let Some(content) = message.message.read().content.clone()
                            && !content.is_empty()
                        {
                            label()
                                .text(content)
                                .max_lines(1)
                                .text_overflow(TextOverflow::Ellipsis)
                                .into_element()
                        } else if has_attachments {
                            label()
                                .font_slant(FontSlant::Italic)
                                .text("Sent an attachment")
                                .into_element()
                        } else {
                            rect().into_element()
                        },
                    ),
            )
            .child(
                Button::new()
                    .child(if self.reply.read().mention {
                        "on"
                    } else {
                        "off"
                    })
                    .on_press({
                        let mut replies = self.replies.clone();
                        let message_id = message.message.peek().id.clone();

                        move |_| {
                            replies.toggle_mention(&message_id);
                        }
                    }),
            )
            .child(Button::new().child("X").on_press({
                let mut replies = self.replies.clone();
                let message_id = message.message.peek().id.clone();

                move |_| {
                    replies.remove_reply(&message_id);
                }
            }))
    }
}
