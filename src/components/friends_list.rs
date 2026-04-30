use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{FriendButton, FriendPage},
};

#[derive(PartialEq)]
pub struct FriendsList {
    pub page: State<FriendPage>,
}

impl Component for FriendsList {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::UserId);
        let user_id = radio.slice_current(|state| state.user_id.as_ref().unwrap());

        let user = radio.slice(AppChannel::Users, move |state| {
            state
                .users
                .get(user_id.read().as_str())
                .inspect(|user| println!("{user:?}"))
                .unwrap()
        });

        let relations = use_memo({
            let page = self.page.clone();
            let radio = radio.clone();

            move || {
                let page = *page.read();

                user.read()
                    .relations
                    .clone()
                    .into_iter()
                    .map::<Readable<v0::User>, _>(|relationship| {
                        radio
                            .slice(AppChannel::Users, {
                                let user_id = relationship.user_id.clone();
                                move |state| state.users.get(&user_id).unwrap()
                            })
                            .into_readable()
                    })
                    .filter(|user| match page {
                        FriendPage::Online => user.read().online,
                        FriendPage::All => true,
                        FriendPage::Pending => [
                            v0::RelationshipStatus::Incoming,
                            v0::RelationshipStatus::Outgoing,
                        ]
                        .contains(&user.read().relationship),
                        FriendPage::Blocked => [
                            v0::RelationshipStatus::Blocked,
                            v0::RelationshipStatus::BlockedOther,
                        ]
                        .contains(&user.read().relationship),
                    })
                    .collect::<Vec<_>>()
            }
        });

        rect().padding((0., 16.)).child(
            VirtualScrollView::new(move |idx, _| {
                let user = relations.read()[idx].clone();

                rect()
                    .key(user.peek().id.clone())
                    .padding((0., 0., 2., 0.))
                    .child(FriendButton { user })
                    .into_element()
            })
            .item_size(54.)
            .length(relations.read().len()),
        )
    }
}
