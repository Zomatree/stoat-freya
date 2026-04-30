use std::{borrow::Cow, collections::HashMap};

use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{StoatButton, avatar},
    http,
};

#[derive(Clone)]
enum ListValue {
    Name(String, usize),
    Member(Readable<v0::User>, Readable<v0::Member>),
}

impl PartialEq for ListValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Name(name0, length0), Self::Name(name1, length1)) => name0 == name1 && length0 == length1,
            (Self::Member(user0, _), Self::Member(user1, _)) => user0.peek().id == user1.peek().id,
            _ => false,
        }
    }
}

#[derive(PartialEq)]
pub struct MemberList {
    pub server: Readable<v0::Server>,
}

impl Component for MemberList {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Members);

        let slice = radio.slice_current({
            let server = self.server.clone();
            move |state| state.members.get(&server.peek().id).unwrap()
        });

        use_future({
            let radio = radio.clone();
            let server = self.server.clone();

            move || {
                let mut radio = radio.clone();
                let server = server.clone();

                async move {
                    let server_id = server.peek().id.clone();

                    let response = http()
                        .fetch_server_members(
                            &server_id,
                            &v0::OptionsFetchAllMembers {
                                exclude_offline: Some(true),
                            },
                        )
                        .await
                        .unwrap();

                    let mut state = radio.write_channel(AppChannel::Users);

                    for user in response.users {
                        state.users.insert(user.id.clone(), user);
                    }

                    drop(state);

                    let mut state = radio.write_channel(AppChannel::Members);
                    let server_members = state.members.get_mut(&server_id).unwrap();

                    for member in response.members {
                        server_members.insert(member.id.user.clone(), member);
                    }
                }
            }
        });

        let hoisted_roles = use_memo({
            let server = self.server.clone();
            move || {
                let mut roles = server
                    .read()
                    .roles
                    .values()
                    .filter(|role| role.hoist)
                    .cloned()
                    .collect::<Vec<_>>();
                roles.sort_by(|a, b| a.rank.cmp(&b.rank));
                roles
            }
        });

        let groups = use_memo({
            let slice = slice.clone();

            move || {
                let members = slice.read();
                let roles = hoisted_roles.read();

                let mut groups = HashMap::new();
                groups.insert("default".to_string(), Vec::new());

                for role in roles.iter() {
                    groups.insert(role.id.clone(), Vec::new());
                }

                'a: for member in members.values() {
                    let user_slice = radio.slice(AppChannel::Users, {
                        let user_id = member.id.user.clone();
                        move |state| state.users.get(&user_id).unwrap()
                    });
                    let user = user_slice.read();

                    if !user.online {
                        continue;
                    };

                    if !member.roles.is_empty() {
                        for hoisted_role in roles.iter() {
                            if member.roles.contains(&hoisted_role.id) {
                                groups
                                    .get_mut(&hoisted_role.id)
                                    .unwrap()
                                    .push(member.id.user.clone());
                                continue 'a;
                            }
                        }
                    };

                    groups
                        .get_mut("default")
                        .unwrap()
                        .push(member.id.user.clone());
                }

                let mut out = Vec::new();

                for role in roles.iter() {
                    let members = groups.remove(&role.id).unwrap();

                    if !members.is_empty() {
                        out.push((role.name.clone(), members));
                    };
                }

                let default = groups.remove("default").unwrap();

                if !members.is_empty() {
                    out.push(("Online".to_string(), default));
                };

                out
            }
        });

        let elements = use_memo({
            let groups = groups.clone();
            let radio = radio.clone();
            let server_id = self.server.peek().id.clone();

            move || {
                let mut elements = Vec::new();

                for (title, members) in groups.read().iter() {
                    elements.push(ListValue::Name(title.clone(), members.len()));

                    for user_id in members.clone() {
                        let user = radio.slice(AppChannel::Users, {
                            let user_id = user_id.clone();
                            move |state| state.users.get(&user_id).unwrap()
                        });

                        let member = radio.slice(AppChannel::Members, {
                            let server_id = server_id.clone();
                            move |state| {
                                state
                                    .members
                                    .get(&server_id)
                                    .unwrap()
                                    .get(&user_id)
                                    .unwrap()
                            }
                        });

                        elements.push(ListValue::Member(
                            user.into_readable(),
                            member.into_readable(),
                        ));
                    }
                };

                elements
            }
        });

        rect().child(VirtualScrollView::new({
            let elements = elements.clone();
            let server = self.server.clone();

            move |i, _| {
                let element = elements.read()[i].clone();

                match element {
                    ListValue::Name(name, count) => {
                        rect()
                            .key(&name)
                            .height(Size::px(42.))
                            .padding((0., 14.))
                            .main_align(Alignment::End)
                            .child(label().text(format!("{name} - {count}")).font_size(11.))
                            .into_element()
                    },
                    ListValue::Member(user, member) => {
                        MemberListMember {
                            server: server.clone(),
                            member,
                            user,
                        }.into_element()
                    }
                }
            }
        }).item_size(42.).length(elements.read().len()))

        // rect().child(
        //     VirtualScrollView::new({
        //         let server = self.server.clone();

        //         move |i, _| {
        //             if i == 0 {

        //             } else {
        //                 let user_id = members.read()[i - 1].clone();

        //                 let member = map_readable::<HashMap<String, v0::Member>, v0::Member>(
        //                     slice.clone().into_readable(),
        //                     {
        //                         let user_id = user_id.clone();
        //                         move |members| members.get(&user_id).unwrap()
        //                     },
        //                 );

        //                 let user = radio.slice(AppChannel::Users, move |state| {
        //                     state.users.get(&user_id).unwrap()
        //                 });

        //                 MemberListMember {
        //                     server: server.clone(),
        //                     member,
        //                     user: user.into_readable(),
        //                 }
        //                 .into_element()
        //             }
        //         }
        //     })
        //     .item_size(42.)
        //     .length(members.read().len() + 1)
        // )
    }
}

#[derive(PartialEq)]
pub struct MemberListMember {
    pub server: Readable<v0::Server>,
    pub member: Readable<v0::Member>,
    pub user: Readable<v0::User>,
}

impl Component for MemberListMember {
    fn render(&self) -> impl IntoElement {
        // let user = use_memo({
        //     let user = self.user.clone();
        //     move || user.read().clone()
        // });

        // let member = use_memo()

        rect()
            .padding((0., 16., 0., 4.))
            .height(Size::px(42.))
            .child(
                StoatButton::new().child(
                    rect()
                        .padding((0., 8.))
                        .horizontal()
                        .height(Size::Fill)
                        .cross_align(Alignment::Center)
                        .spacing(8.)
                        .child(
                            avatar(&self.user.read(), Some(&self.member.read()))
                                .width(Size::px(32.))
                                .height(Size::px(32.)),
                        )
                        .child(
                            rect()
                                .child(
                                    label()
                                        .text({
                                            let user = self.user.read();
                                            let member = self.member.read();
                                            member
                                                .nickname
                                                .as_ref()
                                                .or(user.display_name.as_ref())
                                                .unwrap_or(&user.username)
                                                .clone()
                                        })
                                        .font_size(14)
                                        .max_lines(1)
                                        .text_overflow(TextOverflow::Ellipsis),
                                )
                                .maybe_child(
                                    self.user
                                        .read()
                                        .status
                                        .as_ref()
                                        .and_then(|status| {
                                            status
                                                .text
                                                .as_ref()
                                                .map(|text| Cow::Owned(text.clone()))
                                                .or(status.presence.as_ref().map(|presence| {
                                                    match presence {
                                                        v0::Presence::Online => {
                                                            Cow::Borrowed("Online")
                                                        }
                                                        v0::Presence::Idle => Cow::Borrowed("Idle"),
                                                        v0::Presence::Focus => {
                                                            Cow::Borrowed("Focus")
                                                        }
                                                        v0::Presence::Busy => Cow::Borrowed("Busy"),
                                                        v0::Presence::Invisible => {
                                                            Cow::Borrowed("Invisible")
                                                        }
                                                    }
                                                }))
                                        })
                                        .map(|text| {
                                            label()
                                                .text(text)
                                                .font_size(11)
                                                .max_lines(1)
                                                .text_overflow(TextOverflow::Ellipsis)
                                        }),
                                ),
                        ),
                ),
            )
    }

    fn render_key(&self) -> DiffKey {
        (&self.member.peek().id).into()
    }
}
