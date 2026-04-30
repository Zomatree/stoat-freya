use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{MessageModel, NetworkSvg, StoatButton},
    http,
    types::Tag,
};

#[derive(PartialEq)]
pub struct MessageReactions {
    pub message: MessageModel,
    pub channel: Readable<v0::Channel>,
}

impl Component for MessageReactions {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::UserId);

        let user_id = radio.slice_current(|state| state.user_id.as_ref().unwrap());

        rect()
            .spacing(4.)
            .horizontal()
            .content(Content::wrap_spacing(4.))
            .children(
                self.message
                    .message
                    .read()
                    .reactions
                    .clone()
                    .into_iter()
                    .map(|(emoji, users)| {
                        StoatButton::new()
                        .on_press({
                            let channel = self.channel.clone();
                            let message = self.message.clone();
                            let emoji = emoji.clone();
                            let users = users.clone();
                            let user_id = user_id.clone();
                            let users = users.clone();

                            move |_| {
                                let channel = channel.clone();
                                let message = message.clone();
                                let user_id = user_id.clone();
                                let emoji = emoji.clone();
                                let users = users.clone();

                                spawn(async move {
                                    let channel_id = channel.read().id().to_string();
                                    let message_id = message.message.read().id.clone();
                                    let reacted = users.contains(&*user_id.read());

                                    if reacted {
                                    http().unreact_message(&channel_id, &message_id, &emoji, &v0::OptionsUnreact {
                                        user_id: None,
                                        remove_all: None,
                                    }).await.unwrap();
                                } else {
                                    http().react_message(&channel_id, &message_id, &emoji).await.unwrap();
                                }
                            });
                            }
                        })
                        .child(
                        rect()
                            .key(&emoji)
                            .horizontal()
                            .corner_radius(12.)
                            .padding(8.)
                            .spacing(8.)
                            .background(0xff1b1b21)
                            .maybe(users.contains(&*user_id.read()), |this| this.background(0xff424659))
                            .child({
                                if emoji.len() == 26 {
                                    let url = format!(
                                        "{}/{}/{emoji}",
                                        http().api_config.features.autumn.url,
                                        Tag::Emojis
                                    );

                                    ImageViewer::new(url.parse::<Uri>().unwrap())
                                        .sampling_mode(SamplingMode::Trilinear)
                                        .width(Size::px(18.))
                                        .height(Size::px(18.))
                                        .into_element()
                                } else {
                                    let codes = emoji
                                        .chars()
                                        .map(|c| format!("{:x}", c as i32))
                                        .collect::<Vec<String>>()
                                        .join("-");

                                    let url = format!(
                                        "https://static.stoat.chat/emoji/fluent-3d/{codes}.svg?v=1"
                                    );

                                    NetworkSvg::new(url.parse::<Uri>().unwrap())
                                        .width(Size::px(18.))
                                        .height(Size::px(18.))
                                        .into_element()
                                }
                            })
                            .child(label().text(users.len().to_string()).font_size(14))
                              )      .into_element()
                    }),
            )
    }
}
