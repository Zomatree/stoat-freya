use freya::{prelude::*, radio::use_radio};
use jiff::{Timestamp, tz::TimeZone};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{Avatar, MessageContent, MessageModel, MessageReply},
    member_display_color,
};

#[derive(PartialEq)]
pub struct Message {
    pub channel: Readable<v0::Channel>,
    pub message: MessageModel,
}

impl Component for Message {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Servers);

        let message = self.message.message.read().clone();

        let server = use_memo({
            let member = self.message.member.clone();

            move || {
                if let Some(member) = &member {
                    let member = member.peek();

                    radio.read().servers.get(&member.id.server).cloned()
                } else {
                    None
                }
            }
        });

        let role_color = use_memo({
            let member = self.message.member.clone();

            move || {
                if let Some(member) = &member
                    && let Some(server) = &*server.read()
                {
                    member_display_color(&member.read(), server)
                } else {
                    None
                }
            }
        });

        let display_name = use_memo({
            let user = self.message.user.clone();
            let member = self.message.member.clone();

            move || {
                member
                    .as_ref()
                    .and_then(|member| member.read().nickname.clone())
                    .unwrap_or_else(|| {
                        let user = user.read();

                        user.display_name.as_ref().unwrap_or(&user.username).clone()
                    })
            }
        });

        rect()
            .maybe_child((!self.message.replies.is_empty()).then(|| {
                rect().children(self.message.replies.iter().cloned().map(|(id, reply)| {
                    rect()
                        .key(id)
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
                    .horizontal()
                    .spacing(8.)
                    .child(
                        rect()
                            .horizontal()
                            .main_align(Alignment::End)
                            .width(Size::px(54.))
                            .padding((2., 4.))
                            .child(Avatar::new(
                                self.message.user.clone(),
                                self.message.member.clone(),
                                36.,
                            )),
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
                                    // .background(background)
                                    .child(
                                        label()
                                            .text(display_name.read().clone())
                                            .map(role_color.read().clone(), |this, color| {
                                                if let Fill::Color(color) = color {
                                                    this.color(color)
                                                } else {
                                                    this
                                                }
                                            })
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
                                            .color(0xff90909a)
                                            .font_size(12),
                                    )
                                    .maybe_child(message.edited.as_ref().map(|_ts| {
                                        label()
                                            .text("(edited)")
                                            .font_size(12)
                                            .color(0xff90909a)
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
