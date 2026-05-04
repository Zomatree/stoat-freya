use std::{borrow::Cow, collections::HashMap};

use freya::{icons::lucide::plus, prelude::*, radio::use_radio, text_edit::*};
use rfd::AsyncFileDialog;
use stoat_models::v0;

use crate::{AppChannel, LocalFile, components::ReplyController, http};

#[derive(PartialEq)]
pub struct Textbox {
    pub replies: ReplyController,
    pub attachments: State<HashMap<u64, (String, Bytes)>>,
    pub channel: Readable<v0::Channel>,
}

impl Component for Textbox {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::UserId);
        let holder = use_state(ParagraphHolder::default);
        let mut editable = use_editable(String::new, EditableConfig::new);
        let focus = use_focus();

        rect()
            .font_size(14)
            .horizontal()
            .min_height(Size::px(40.))
            .spacing(8.)
            .content(Content::Fit)
            .child(
                rect()
                    .horizontal()
                    .background(0xff292a2f)
                    .corner_radius(CornerRadius {
                        top_left: 28.,
                        top_right: 28.,
                        bottom_right: 28.,
                        bottom_left: 28.,
                        smoothing: 0.,
                    })
                    .padding((4., 0.))
                    .cross_align(Alignment::Center)
                    .child(
                        rect().width(Size::px(62.)).center().child(
                            Button::new()
                                .color(0xffc6c5d0)
                                .width(Size::px(40.))
                                .height(Size::px(40.))
                                .flat()
                                .on_press({
                                    let mut attachments = self.attachments.clone();

                                    move |_| {
                                        spawn(async move {
                                            if let Some(file) =
                                                AsyncFileDialog::new().pick_file().await
                                            {
                                                let contents = file.read().await.into();
                                                let filename = file.file_name();

                                                attachments
                                                    .write()
                                                    .insert(rand::random(), (filename, contents));
                                            };
                                        });
                                    }
                                })
                                .child(svg(plus()).width(Size::px(24.)).height(Size::px(24.))),
                        ),
                    )
                    .child(
                        rect()
                            .child(
                                paragraph()
                                    .margin((4., 2., 4., 6.))
                                    // .width(Size::Fill)
                                    .width(Size::func(|size| Some(size.parent - 16.)))
                                    .a11y_id(focus.a11y_id())
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
                                        focus.request_focus();
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
                                        let mut attachments = self.attachments.clone();

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

                                                let attachments =
                                                    std::mem::take(&mut *attachments.write());

                                                spawn({
                                                    let channel_id =
                                                        channel.clone().read().id().to_string();

                                                    async move {
                                                        let mut attachment_ids = Vec::new();

                                                        for (name, content) in
                                                            attachments.into_values()
                                                        {
                                                            let file = http()
                                                                .upload_file(
                                                                    "attachments",
                                                                    LocalFile {
                                                                        name,
                                                                        body: content.into(),
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
                    ),
            )
    }
}
