use freya::{
    icons::lucide::{at_sign, circle_x, file_text},
    prelude::*,
};
use stoat_models::v0;

use crate::components::{Avatar, ReplyController, ReplyIntent, StoatButton};

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
            .content(Content::Flex)
            .font_size(14)
            .spacing(4.)
            .cross_align(Alignment::Center)
            .child(label().font_size(12).text("Replying to"))
            .child(
                rect()
                    .horizontal()
                    .spacing(4.)
                    .cross_align(Alignment::Center)
                    .width(Size::flex(1.))
                    .child(Avatar::new(
                        message.user.clone(),
                        message.member.clone(),
                        14.,
                    ))
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
                rect()
                    .horizontal()
                    .spacing(15.)
                    .cross_align(Alignment::Center)
                    .main_align(Alignment::End)
                    .child({
                        let mention = self.reply.read().mention;

                        StoatButton::new()
                            .child(
                                rect()
                                    .spacing(4.)
                                    .horizontal()
                                    .cross_align(Alignment::Center)
                                    .color(if mention { 0xffdde1ff } else { 0xff90909a })
                                    .child(
                                        svg(at_sign()).width(Size::px(16.)).height(Size::px(16.)),
                                    )
                                    .child(if mention {
                                        "ON"
                                    } else {
                                        "OFF"
                                    }),
                            )
                            .on_press({
                                let mut replies = self.replies.clone();
                                let message_id = message.message.peek().id.clone();

                                move |_| {
                                    replies.toggle_mention(&message_id);
                                }
                            })
                    })
                    .child(
                        StoatButton::new()
                            .child(
                                svg(circle_x())
                                    .color(0xffdde1ff)
                                    .width(Size::px(16.))
                                    .height(Size::px(16.)),
                            )
                            .on_press({
                                let mut replies = self.replies.clone();
                                let message_id = message.message.peek().id.clone();

                                move |_| {
                                    replies.remove_reply(&message_id);
                                }
                            }),
                    ),
            )
    }
}
