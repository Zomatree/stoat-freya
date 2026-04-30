use freya::prelude::*;
use jiff::{Timestamp, tz::TimeZone};
use stoat_models::v0;

use crate::components::{MessageContent, MessageModel, MessageReply, avatar::avatar};

#[derive(PartialEq)]
pub struct Message {
    pub channel: Readable<v0::Channel>,
    pub message: MessageModel,
}

impl Component for Message {
    fn render(&self) -> impl IntoElement {
        let message = self.message.message.read().clone();

        rect()
            .maybe_child((!self.message.replies.is_empty()).then(|| {
                rect().children(self.message.replies.iter().cloned().map(|reply| {
                    rect()
                        .key(reply.message.peek().id.clone())
                        .child(MessageReply {
                            channel: self.channel.clone(),
                            message: self.message.clone(),
                            reply,
                        })
                        .into_element()
                }))
            }))
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .spacing(8.)
                    .child(
                        rect()
                            .horizontal()
                            .main_align(Alignment::End)
                            .width(Size::px(54.))
                            .padding((2., 4.))
                            .child(
                                self.message
                                    .member
                                    .as_ref()
                                    .map(|r| avatar(&self.message.user.read(), Some(&r.read())))
                                    .unwrap_or_else(|| avatar(&self.message.user.read(), None))
                                    .width(Size::px(36.))
                                    .height(Size::px(36.)),
                            ),
                    )
                    .child(
                        rect()
                            .spacing(2.)
                            .child(
                                rect()
                                    .horizontal()
                                    .spacing(8.)
                                    .cross_align(Alignment::Center)
                                    .font_size(14)
                                    .child(
                                        label()
                                            .text(
                                                self.message
                                                    .member
                                                    .as_ref()
                                                    .and_then(|member| {
                                                        member.read().nickname.clone()
                                                    })
                                                    .or_else(|| {
                                                        self.message
                                                            .user
                                                            .read()
                                                            .display_name
                                                            .clone()
                                                    })
                                                    .unwrap_or_else(|| {
                                                        self.message.user.read().username.clone()
                                                    }),
                                            )
                                            .line_height(1.5),
                                    )
                                    .child(
                                        label()
                                            .text({
                                                let datetime = Timestamp::try_from(
                                                    ulid::Ulid::from_string(
                                                        &self.message.message.read().id,
                                                    )
                                                    .unwrap()
                                                    .datetime(),
                                                )
                                                .unwrap()
                                                .to_zoned(TimeZone::system());

                                                let now =
                                                    Timestamp::now().to_zoned(TimeZone::system());

                                                if datetime.date() == now.date() {
                                                    format!(
                                                        "Today at {:02}:{:02}",
                                                        datetime.hour(),
                                                        datetime.minute()
                                                    )
                                                } else if now.date().yesterday().unwrap()
                                                    == datetime.date()
                                                {
                                                    format!(
                                                        "Yesterday at {:02}:{:02}",
                                                        datetime.hour(),
                                                        datetime.minute()
                                                    )
                                                } else {
                                                    format!(
                                                        "{:02}/{:02}/{}",
                                                        datetime.day(),
                                                        datetime.month(),
                                                        datetime.year()
                                                    )
                                                }
                                            })
                                            .color(Color::DARK_GRAY)
                                            .font_size(12),
                                    )
                                    .maybe_child(message.edited.as_ref().map(|_ts| {
                                        label()
                                            .text("(edited)")
                                            .font_size(12)
                                            .color(Color::DARK_GRAY)
                                    })),
                            )
                            .child(MessageContent {
                                channel: self.channel.clone(),
                                message: self.message.clone(),
                            }),
                    ),
            )
    }
}
