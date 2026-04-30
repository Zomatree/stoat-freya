use freya::{
    icons::lucide::{ellipsis_vertical, pencil, smile, trash_2, undo},
    prelude::*,
    radio::use_radio,
};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{MessageModel, ReplyController},
};

#[derive(PartialEq)]
pub struct MessageActions {
    pub children: Vec<Element>,
    pub replies: ReplyController,
    pub channel: Readable<v0::Channel>,
    pub message: MessageModel,
}

impl MessageActions {
    pub fn new(
        replies: ReplyController,
        channel: Readable<v0::Channel>,
        message: MessageModel,
    ) -> Self {
        Self {
            children: Vec::new(),
            replies,
            channel,
            message,
        }
    }
}

impl ChildrenExt for MessageActions {
    fn get_children(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}

impl Component for MessageActions {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::UserId);
        let user_id = radio.slice_current(|state| state.user_id.as_ref().unwrap());

        let mut hovering = use_state(|| false);
        let mut hover_actions = use_state(|| false);

        let mentions_user = use_memo({
            let message = self.message.message.clone();

            move || {
                message
                    .read()
                    .mentions
                    .as_ref()
                    .is_some_and(|mentions| mentions.contains(&*user_id.read()))
            }
        });

        rect()
            .width(Size::Fill)
            .on_pointer_over(move |_| {
                hovering.set(true);
            })
            .on_pointer_out(move |_| hovering.set_if_modified(false))
            .on_secondary_down({
                let message = self.message.clone();
                let replies = self.replies.clone();

                move |e| {
                    ContextMenu::open_from_event(
                        &e,
                        Menu::new()
                            .child(MenuButton::new().child("Copy text").on_press({
                                let message = message.clone();
                                move |_| {
                                    if let Some(content) = message.message.read().content.clone() {
                                        Clipboard::set(content).unwrap();
                                    }
                                }
                            }))
                            .child(MenuButton::new().child("Reply").on_press({
                                let message = message.clone();
                                let mut replies = replies.clone();

                                move |_| {
                                    replies.add_reply(message.clone(), true);
                                }
                            })),
                    );
                }
            })
            .corner_radius(12.)
            // .overflow(Overflow::Clip)
            .children(self.children.clone())
            .maybe(*mentions_user.read(), |this| this.background(0xff384379))
            .maybe(*hovering.read() || *hover_actions.read(), |this| {
                if *mentions_user.read() {
                    this.background(0xff1f1f25)
                } else {
                    this.background(0xff1f1f25)
                }
            })
            .maybe_child((*hovering.read() || *hover_actions.read()).then(|| {
                rect()
                    .on_pointer_over(move |_| {
                        hover_actions.set(true);
                    })
                    .on_pointer_out(move |_| hover_actions.set_if_modified(false))
                    .position(Position::new_absolute().right(16.).top(-18.))
                    .corner_radius(4.)
                    .overflow(Overflow::Clip)
                    .horizontal()
                    .shadow(Shadow::new().blur(3.).color(Color::BLACK))
                    .child(message_actions_button(undo()).on_press({
                        let mut replies = self.replies;
                        let message = self.message.clone();

                        move |_| {
                            replies.add_reply(message.clone(), true);
                        }
                    }))
                    .child(message_actions_button(smile()))
                    .child(message_actions_button(pencil()))
                    .child(message_actions_button(trash_2()))
                    .child(message_actions_button(ellipsis_vertical()))
            }))
    }

    fn render_key(&self) -> DiffKey {
        (&self.message.message.peek().id).into()
    }
}

pub fn message_actions_button(icon: Bytes) -> Button {
    Button::new()
        .flat()
        .corner_radius(0.)
        .child(
            svg(icon)
                .color(0xffdfe1f9)
                .width(Size::px(20.))
                .height(Size::px(20.)),
        )
        .padding(4.)
        .background(0xff424659)
}
