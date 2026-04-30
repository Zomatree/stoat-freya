use freya::prelude::*;
use stoat_models::v0;

use crate::components::avatar;

#[derive(PartialEq)]
pub struct FriendButton {
    pub user: Readable<v0::User>,
}

impl Component for FriendButton {
    fn render(&self) -> impl IntoElement {
        let mut hovering = use_state(|| false);

        let user = self.user.read();

        rect()
            .horizontal()
            .padding((8., 16.))
            .width(Size::Fill)
            .maybe(*hovering.read(), |this| this.background(0xbb000000))
            .cross_align(Alignment::Center)
            .spacing(16.)
            .on_pointer_over(move |_| {
                hovering.set(true);
            })
            .on_pointer_out(move |_| hovering.set_if_modified(false))
            .child(
                avatar(&user, None)
                    .width(Size::px(36.))
                    .height(Size::px(36.)),
            )
            .child(user.username.clone())
    }
}
