use std::{borrow::Cow, fmt::Debug, mem};

use freya::{
    icons::lucide::{at_sign, hash, notebook_text, pin, users_round},
    prelude::*,
    radio::use_radio,
};
use indexmap::IndexMap;
use rfd::AsyncFileDialog;
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{
        ChannelMessages, HideSidebarHeader, MemberList, MessageAttachmentsPreview, MessageModel,
        MessageReplyPreview, StoatButton, StoatButtonLayoutThemePartialExt, StoatTooltip, Textbox,
    },
    map_readable, use_config, use_material_theme,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ReplyIntent {
    pub message: MessageModel,
    pub mention: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub struct ReplyController(State<Vec<ReplyIntent>>);

impl ReplyController {
    pub fn get_replies(&self) -> impl Iterator<Item = Readable<ReplyIntent>> {
        self.0.read().clone().into_iter().map(|reply| {
            let id = reply.message.message.id.clone();

            map_readable::<Vec<ReplyIntent>, ReplyIntent>(self.0.into_readable(), move |replies| {
                replies.iter().find(|r| r.message.message.id == id).unwrap()
            })
        })
    }

    pub fn toggle_mention(&mut self, message_id: &str) {
        if let Some(reply) = self
            .0
            .write()
            .iter_mut()
            .find(|r| r.message.message.id == message_id)
        {
            reply.mention = !reply.mention;
        }
    }

    pub fn add_reply(&mut self, message: MessageModel, mention: bool) {
        let message_id = &message.message.id;
        let mut replies = self.0.write();

        if replies
            .iter()
            .any(|reply| &reply.message.message.id == message_id)
        {
            return;
        };

        replies.push(ReplyIntent { message, mention });
    }

    pub fn remove_reply(&mut self, message_id: &str) {
        self.0.with_mut(|mut replies| {
            replies.retain(|r| r.message.message.id != message_id);
        });
    }

    pub fn take_replies(&mut self) -> Vec<v0::ReplyIntent> {
        let replies = std::mem::take(&mut *self.0.write());

        replies
            .into_iter()
            .map(|reply| v0::ReplyIntent {
                id: reply.message.message.id.clone(),
                mention: reply.mention,
                fail_if_not_exists: Some(true),
            })
            .collect()
    }
}

#[derive(Clone, PartialEq)]
pub struct Attachment {
    pub controller: AttachmentController,

    pub id: u64,
    pub filename: String,
    pub spoiler: bool,
    pub contents: Bytes,
}

impl Debug for Attachment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Attachment")
            .field("id", &self.id)
            .field("filename", &self.filename)
            .field("spoiler", &self.spoiler)
            .field("contents", &self.contents)
            .finish_non_exhaustive()
    }
}

impl Attachment {
    pub fn remove(&self) {
        self.controller.remove(self.id);
    }

    pub fn toggle_spoiler(&self) {
        self.controller.toggle_spoiler(self.id);
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct AttachmentController(State<IndexMap<u64, Attachment>>);

impl AttachmentController {
    pub fn is_empty(&self) -> bool {
        self.0.read().is_empty()
    }

    pub fn not_empty(&self) -> bool {
        !self.is_empty()
    }

    pub fn get_attachments(&self) -> impl Iterator<Item = Attachment> {
        self.0.read().clone().into_values()
    }

    pub fn remove(&self, id: u64) {
        self.0.clone().write().shift_remove(&id);
    }

    pub fn take(&self) -> IndexMap<u64, Attachment> {
        mem::take(&mut *self.0.clone().write())
    }

    pub fn toggle_spoiler(&self, id: u64) {
        self.0.clone().with_mut(|mut attachments| {
            if let Some(attachment) = attachments.get_mut(&id) {
                attachment.spoiler = !attachment.spoiler;
            };
        });
    }

    pub async fn prompt(&self) {
        if let Some(file) = AsyncFileDialog::new().pick_file().await {
            let contents = file.read().await.into();
            let filename = file.file_name();

            let id = rand::random();

            let (filename, spoiler) = if let Some(filename) = filename.strip_prefix("SPOILER_") {
                (filename.to_string(), true)
            } else {
                (filename, false)
            };

            let attachment = Attachment {
                controller: *self,
                id,
                filename,
                spoiler,
                contents,
            };

            self.0.clone().write().insert(id, attachment);
        };
    }
}

#[derive(PartialEq)]
pub struct Channel {
    pub channel: Readable<v0::Channel>,
    pub server: Option<Readable<v0::Server>>,
}

impl Component for Channel {
    fn render(&self) -> impl IntoElement {
        let mut config = use_config();
        let radio = use_radio(AppChannel::UserId);
        let theme = use_material_theme();

        let mut textbox_size = use_state(Area::default);

        let replies = ReplyController(use_state(Vec::<ReplyIntent>::new));
        let attachments = AttachmentController(use_state(IndexMap::new));

        let hide_members_list = config.read().hide_members_list;

        let search = use_state(String::new);

        rect()
            .child(
                rect()
                    .horizontal()
                    .height(Size::px(48.))
                    .padding((0., 24., 0., 8.))
                    .margin((8., 0.))
                    .spacing(10.)
                    .cross_align(Alignment::Center)
                    .content(Content::Flex)
                    .child(HideSidebarHeader {
                        icon: match &*self.channel.read() {
                            v0::Channel::DirectMessage { .. } => at_sign(),
                            v0::Channel::SavedMessages { .. } => notebook_text(),
                            _ => hash(),
                        },
                    })
                    .child(
                        label()
                            .text(match &*self.channel.read() {
                                v0::Channel::DirectMessage { recipients, .. } => {
                                    let user_id = radio.peek_state().user_id.clone().unwrap();

                                    let other = recipients
                                        .iter()
                                        .find(|&id| id != &*user_id)
                                        .unwrap()
                                        .clone();

                                    let user = radio.slice(AppChannel::Users, move |state| {
                                        state.users.get(&other).unwrap()
                                    });

                                    Cow::Owned(user.read().username.clone())
                                }
                                v0::Channel::Group { name, .. }
                                | v0::Channel::TextChannel { name, .. } => Cow::Owned(name.clone()),
                                v0::Channel::SavedMessages { .. } => {
                                    Cow::Borrowed("Saved Messages")
                                }
                            })
                            .font_size(16)
                            .max_lines(1),
                    )
                    .child(rect().width(Size::flex(1.)))
                    .child(
                        StoatTooltip::new(
                            label()
                                .font_size(11.)
                                .max_lines(1)
                                .text("View pinned messages"),
                        )
                        .position(AttachedPosition::Bottom)
                        .child(
                            StoatButton::new()
                                .corner_radius(40.)
                                .on_press(move |_| {})
                                .child(
                                    rect()
                                        .horizontal()
                                        .height(Size::px(40.))
                                        .padding((0., 8.))
                                        .center()
                                        .color(theme.md.on_surface_variant.as_argb_u32())
                                        .child(
                                            svg(pin()).width(Size::px(24.)).height(Size::px(24.)),
                                        ),
                                ),
                        ),
                    )
                    .child(
                        StoatTooltip::new(label().font_size(11.).max_lines(1).text("View members"))
                            .position(AttachedPosition::Bottom)
                            .child(
                                StoatButton::new()
                                    .corner_radius(40.)
                                    .on_press(move |_| {
                                        config.write().hide_members_list = !hide_members_list
                                    })
                                    .child(
                                        rect()
                                            .horizontal()
                                            .height(Size::px(40.))
                                            .padding((0., 8.))
                                            .center()
                                            .color(theme.md.on_surface_variant.as_argb_u32())
                                            .child(
                                                svg(users_round())
                                                    .width(Size::px(24.))
                                                    .height(Size::px(24.)),
                                            ),
                                    ),
                            ),
                    )
                    .child(
                        rect()
                            .child(
                                Input::new(search)
                                    .placeholder("Search messages...")
                                    .border_fill(Color::TRANSPARENT)
                                    .inner_margin((10., 16.))
                                    .corner_radius(40.)
                                    .width(Size::Fill)
                                    .background(theme.md.surface_container_high.as_argb_u32()),
                            )
                            .max_width(Size::px(240.)),
                    ),
            )
            .child(
                rect()
                    .height(Size::Fill)
                    .horizontal()
                    .content(Content::Flex)
                    .child(
                        rect()
                            .height(Size::Fill)
                            .width(Size::flex(1.))
                            .content(Content::Flex)
                            .margin((0., 8., 8., 8.))
                            .corner_radius(28.)
                            .background(theme.md.surface_container_lowest.as_argb_u32())
                            .overflow(Overflow::Clip)
                            .child(rect().height(Size::flex(1.)).child(ChannelMessages {
                                replies,
                                channel: self.channel.clone(),
                                server: self.server.clone(),
                            }))
                            .child(
                                rect()
                                    .width(Size::Fill)
                                    .maybe_child(attachments.not_empty().then(|| {
                                        rect()
                                            .margin((0., 0., 8., 0.))
                                            .width(Size::func(|size| Some(size.parent - 16.)))
                                            .child(MessageAttachmentsPreview {
                                                attachments: attachments.clone(),
                                            })
                                            .into_element()
                                    }))
                                    .child(rect().children(replies.get_replies().map(|reply| {
                                        rect()
                                            .key(&reply.read().message.message.id)
                                            .margin((0., 0., 8., 0.))
                                            .child(MessageReplyPreview {
                                                replies,
                                                reply,
                                                channel: self.channel.clone(),
                                            })
                                            .into_element()
                                    })))
                                    .child(Textbox {
                                        replies,
                                        attachments,
                                        channel: self.channel.clone(),
                                    })
                                    .margin((0., 8., 8., 8.))
                                    .on_sized(move |e: Event<SizedEventData>| {
                                        textbox_size.set(e.area)
                                    }),
                            ),
                    )
                    .maybe_child(self.server.as_ref().filter(|_| !hide_members_list).map(
                        |server| {
                            rect()
                                .child(MemberList {
                                    server: server.clone(),
                                })
                                .width(Size::px(248.))
                                .padding((0., 8., 0., 0.))
                        },
                    )),
            )
    }

    fn render_key(&self) -> DiffKey {
        (&self.channel.peek().id().to_string()).into()
    }
}
