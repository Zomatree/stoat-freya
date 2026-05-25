use std::borrow::Cow;

use freya::{
    icons::lucide::{plus, smile},
    prelude::*,
    radio::use_radio,
    text_edit::*,
};
use stoat_models::v0;

use crate::{
    AppChannel, LocalFile,
    components::{
        AttachmentController, EmojiPicker, ReplyController, StoatButton, StoatButtonLayoutThemePartialExt, use_floating
    },
    http, use_material_theme,
};

#[derive(PartialEq)]
pub struct Textbox {
    pub replies: ReplyController,
    pub attachments: AttachmentController,
    pub channel: Readable<v0::Channel>,
}

impl Component for Textbox {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::UserId);
        let theme = use_material_theme();
        let holder = use_state(ParagraphHolder::default);
        let mut editable = use_editable(String::new, EditableConfig::new);
        let a11y_id = use_a11y();
        let mut floating = use_floating();

        rect()
        .width(Size::Fill)
            .font_size(14)
            .horizontal()
            .min_height(Size::px(40.))
            .spacing(8.)
            // .content(Content::Fit)
            .child(
                rect()
                    .horizontal()
                    .content(Content::Flex)
                    .background(theme.md.surface_container_high.as_argb_u32())
                    .corner_radius(CornerRadius {
                        top_left: 28.,
                        top_right: 28.,
                        bottom_right: 28.,
                        bottom_left: 28.,
                        smoothing: 0.,
                    })
                    .padding((4., 8., 4., 0.))
                    .cross_align(Alignment::Center)
                    .child(
                        rect().width(Size::px(62.)).center().child(
                            StoatButton::new()
                                .corner_radius(40.)
                                .on_press({
                                    let attachments = self.attachments.clone();

                                    move |_| {
                                        spawn(async move {
                                            attachments.prompt().await;
                                        });
                                    }
                                })
                                .child(
                                    rect()
                                        .width(Size::px(40.))
                                        .height(Size::px(40.))
                                        .center()
                                        .child(
                                            svg(plus()).width(Size::px(24.)).height(Size::px(24.)),
                                        ),
                                ),
                        ),
                    )
                    .child(
                        rect()
                        .width(Size::flex(1.))
                            .child(
                                paragraph()
                                    .margin((4., 2., 4., 6.))
                                    .width(Size::Fill)
                                    // .width(Size::func(|size| Some(size.parent - 16.)))
                                    .a11y_id(a11y_id)
                                    .a11y_auto_focus(true)
                                    .cursor_index(editable.editor().read().cursor_pos())
                                    .cursor_style(CursorStyle::Line)
                                    .cursor_color(0xFFFFFFFF)
                                    .highlights(
                                        editable
                                            .editor()
                                            .read()
                                            .get_selection()
                                            .map(|selection| vec![selection])
                                            .unwrap_or_default(),
                                    )
                                    .on_mouse_down(move |e: Event<MouseEventData>| {
                                        a11y_id.request_focus();
                                        editable.process_event(EditableEvent::Down {
                                            location: e.element_location,
                                            editor_line: EditorLine::SingleParagraph,
                                            holder: &holder.read(),
                                        });
                                    })
                                    .on_mouse_move(move |e: Event<MouseEventData>| {
                                        editable.process_event(EditableEvent::Move {
                                            location: e.element_location,
                                            editor_line: EditorLine::SingleParagraph,
                                            holder: &holder.read(),
                                        });
                                    })
                                    .on_global_pointer_press(move |_: Event<PointerEventData>| {
                                        editable.process_event(EditableEvent::Release)
                                    })
                                    .on_key_down({
                                        let channel = self.channel.clone();
                                        let mut replies = self.replies.clone();
                                        let attachments = self.attachments.clone();

                                        move |e: Event<KeyboardEventData>| {
                                            if e.key == Key::Named(NamedKey::Enter)
                                                && !e.modifiers.shift()
                                            {
                                                let editor = editable.editor_mut();
                                                let mut writer = editor.write();
                                                let content = writer.to_string();
                                                *writer = RopeEditor::new(
                                                    String::new(),
                                                    TextSelection::new_cursor(0),
                                                    0,
                                                    writer.editor_history().clone(),
                                                );
                                                drop(writer);

                                                let message_replies = replies.take_replies();

                                                let attachments = attachments.take();

                                                spawn({
                                                    let channel_id =
                                                        channel.clone().read().id().to_string();

                                                    async move {
                                                        let mut attachment_ids = Vec::new();

                                                        for attachment in
                                                            attachments.into_values()
                                                        {
                                                            let file = http()
                                                                .upload_file(
                                                                    "attachments",
                                                                    LocalFile {
                                                                        name: if attachment.spoiler {
                                                                            format!("SPOILER_{}", attachment.filename)
                                                                        } else {
                                                                            attachment.filename
                                                                        },
                                                                        body: attachment.contents.into(),
                                                                    },
                                                                )
                                                                .await
                                                                .unwrap();
                                                            attachment_ids.push(file.id);
                                                        }

                                                        http()
                                                            .send_message(
                                                                &channel_id,
                                                                &v0::DataMessageSend {
                                                                    nonce: None,
                                                                    content: Some(content),
                                                                    attachments: Some(
                                                                        attachment_ids,
                                                                    ),
                                                                    replies: Some(message_replies),
                                                                    embeds: None,
                                                                    masquerade: None,
                                                                    interactions: None,
                                                                    flags: None,
                                                                },
                                                            )
                                                            .await
                                                            .unwrap();
                                                    }
                                                });
                                            } else {
                                                editable.process_event(EditableEvent::KeyDown {
                                                    key: &e.key,
                                                    modifiers: e.modifiers,
                                                });
                                            }
                                        }
                                    })
                                    .on_key_up(move |e: Event<KeyboardEventData>| {
                                        editable
                                            .process_event(EditableEvent::KeyUp { key: &e.key });
                                    })
                                    .span(editable.editor().read().to_string())
                                    .holder(holder.read().clone()),
                            )
                            .maybe_child(editable.editor().read().to_string().is_empty().then(
                                || {
                                    rect()
                                        .child(
                                            label()
                                                .text(format!(
                                                    "Message {}",
                                                    match &*self.channel.read() {
                                                        v0::Channel::DirectMessage {
                                                            recipients,
                                                            ..
                                                        } => {
                                                            let user_id = radio
                                                                .peek_state()
                                                                .user_id
                                                                .clone()
                                                                .unwrap();

                                                            let other = recipients
                                                                .iter()
                                                                .find(|&id| id != &*user_id)
                                                                .unwrap()
                                                                .clone();

                                                            let user = radio.slice(
                                                                AppChannel::Users,
                                                                move |state| {
                                                                    state.users.get(&other).unwrap()
                                                                },
                                                            );

                                                            Cow::Owned(user.read().username.clone())
                                                        }
                                                        v0::Channel::Group { name, .. }
                                                        | v0::Channel::TextChannel {
                                                            name, ..
                                                        } => Cow::Owned(name.clone()),
                                                        v0::Channel::SavedMessages { .. } =>
                                                            Cow::Borrowed("Saved Messages"),
                                                    }
                                                ))
                                                .color(0xff888888),
                                        )
                                        .layer(Layer::RelativeOverlay(1))
                                        .position(Position::new_absolute())
                                        .padding((4., 2., 4., 6.))
                                },
                            )),
                    )
                    .child(
                        StoatButton::new()
                            .corner_radius(40.)
                            .on_press({
                                move |_| {
                                    floating.set(Some(
                                        EmojiPicker::new(move |e| println!("{e:?}")).into_element(),
                                    ));
                                }
                            })
                            .child(
                                rect()
                                    .width(Size::px(40.))
                                    .height(Size::px(40.))
                                    .center()
                                    .child(svg(smile()).width(Size::px(24.)).height(Size::px(24.))),
                            ),
                    ),
            )
    }
}
