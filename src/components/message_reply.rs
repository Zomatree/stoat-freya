use freya::{icons::lucide::file_text, prelude::*};
use stoat_models::v0;

use crate::components::{Avatar, MessageModel};

#[derive(PartialEq)]
pub struct MessageReply {
    pub channel: Readable<v0::Channel>,
    pub message: MessageModel,
    pub reply: Option<MessageModel>,
}

impl Component for MessageReply {
    fn render(&self) -> impl IntoElement {
        rect()
            .height(Size::px(22.))
            .horizontal()
            .font_size(14)
            .spacing(4.)
            .cross_align(Alignment::End)
            .child(
                rect()
                    .height(Size::px(12.))
                    .width(Size::px(22.))
                    .margin((0., 6., 0., 30.))
                    .corner_radius(CornerRadius {
                        top_left: 4.,
                        top_right: 0.,
                        bottom_right: 0.,
                        bottom_left: 0.,
                        smoothing: 0.,
                    })
                    .border(Border::new().fill(0xff45464f).width(BorderWidth {
                        top: 2.,
                        right: 0.,
                        bottom: 0.,
                        left: 2.,
                    })),
            )
            .child(match &self.reply {
                Some(reply) => {
                    let has_attachments = reply
                        .message
                        .read()
                        .attachments
                        .as_ref()
                        .is_some_and(|files| !files.is_empty());

                    rect()
                        .height(Size::px(22.))
                        .horizontal()
                        .spacing(4.)
                        .cross_align(Alignment::Center)
                        .child(
                            rect()
                                .height(Size::px(22.))
                                .horizontal()
                                .spacing(4.)
                                .cross_align(Alignment::Center)
                                .child(Avatar::new(reply.user.clone(), reply.member.clone(), 14.))
                                .child(
                                    label()
                                    .line_height(1.25)
                                        .text({
                                            let user = reply.user.read();

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
                                ),
                        )
                        .maybe_child(
                            has_attachments.then(|| {
                                svg(file_text()).width(Size::px(16.)).height(Size::px(16.))
                            }),
                        )
                        .child(
                            if let Some(content) = reply.message.read().content.clone()
                                && !content.is_empty()
                            {
                                label()
                                    .text(content)
                                    .max_lines(1)
                                    .line_height(1.25)
                                    .text_overflow(TextOverflow::Ellipsis)
                                    .into_element()
                            } else if has_attachments {
                                label()
                                    .line_height(1.25)
                                    .font_slant(FontSlant::Italic)
                                    .text("Sent an attachment")
                                    .into_element()
                            } else {
                                rect().into_element()
                            },
                        )
                }
                None => rect()
                    .font_weight(FontWeight::LIGHT)
                    .child("Unknown Message"),
            })
    }
}
