use std::rc::Rc;

use freya::{
    icons::lucide::{circle_x, ellipsis_vertical},
    prelude::*,
    radio::use_radio,
};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{
        Avatar, StoatButton, StoatButtonColorsThemePartialExt, StoatButtonLayoutThemePartialExt,
        image,
    },
    http, parse_fill,
};

#[derive(PartialEq)]
pub struct UserProfile {
    pub user: Readable<v0::User>,
}

impl Component for UserProfile {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::UserProfile);

        let close_profile = move || radio.clone().write().user_profile = None;

        let user = use_memo({
            let user = self.user.clone();
            move || user.read().clone()
        });

        let profile = use_state(|| None);

        use_future({
            let user_id = self.user.peek().id.clone();

            move || {
                let user_id = user_id.clone();
                let mut profile = profile.clone();

                async move {
                    if let Ok(user_profile) = http().fetch_user_profile(&user_id).await {
                        profile.set(Some(user_profile));
                    }
                }
            }
        });

        rect()
            .expanded()
            .center()
            .background(0xBB000000)
            .on_press(move |_| close_profile())
            .on_global_key_down(move |e: Event<KeyboardEventData>| {
                if e.key == Key::Named(NamedKey::Escape) {
                    close_profile()
                }
            })
            .child(
                rect()
                    .on_press(|e: Event<PressEventData>| e.stop_propagation())
                    .corner_radius(28.)
                    .overflow(Overflow::Clip)
                    .background(0xff292a2f)
                    .width(Size::px(560.))
                    .padding(8.)
                    .child(
                        rect()
                            .padding(8.)
                            .spacing(8.)
                            .content(Content::Flex)
                            .child(ProfileBanner {
                                user: self.user.clone(),
                                profile: profile.into_readable(),
                            })
                            .child(ProfileButtons {
                                user: self.user.clone(),
                            })
                            .child({
                                let status_text = user
                                    .read()
                                    .status
                                    .as_ref()
                                    .and_then(|status| status.text.clone());

                                let badges = user.read().badges;

                                let show_hidden = status_text.is_none() || badges == 0;

                                row()
                                    .content(Content::Flex)
                                    .maybe_child(status_text.map(|text| ProfileStatus { text }))
                                    .maybe_child((badges != 0).then(|| ProfileBadges { badges }))
                                    .child(ProfileJoined {
                                        user: self.user.clone(),
                                        member: None,
                                    })
                                    .maybe_child(show_hidden.then(empty_card))
                            })
                            .maybe_child(
                                profile
                                    .read()
                                    .as_ref()
                                    .and_then(|profile| profile.content.clone())
                                    .map(|bio| ProfileBio { bio }),
                            ),
                    ),
            )
    }
}

fn row() -> Rect {
    rect()
        .horizontal()
        .spacing(8.)
        .content(Content::Flex)
        .width(Size::Fill)
}

fn card(title: &'static str, content: impl IntoElement) -> Rect {
    rect()
        .background(0xff1b1b21)
        .width(Size::flex(1.))
        .height(Size::px(170.))
        .padding(15.)
        .corner_radius(16.)
        .spacing(4.)
        .child(
            label()
                .max_lines(1)
                .text(title)
                .font_size(22.)
                .font_weight(FontWeight::SEMI_BOLD),
        )
        .child(content)
}

fn empty_card() -> Rect {
    rect().width(Size::flex(1.)).height(Size::px(170.))
}

#[derive(PartialEq)]
pub struct ProfileBanner {
    pub user: Readable<v0::User>,
    pub profile: Readable<Option<v0::UserProfile>>,
}

impl Component for ProfileBanner {
    fn render(&self) -> impl IntoElement {
        rect()
            .width(Size::Fill)
            .height(Size::px(120.))
            .corner_radius(28.)
            .overflow(Overflow::Clip)
            .maybe_child(
                self.profile
                    .read()
                    .as_ref()
                    .and_then(|p| p.background.as_ref())
                    .map(|background| {
                        image(background)
                            .aspect_ratio(AspectRatio::Max)
                            .image_cover(ImageCover::Center)
                            .expanded()
                    }),
            )
            .child(
                rect()
                    .padding(15.)
                    .position(Position::new_absolute().top(0.).left(0.))
                    .layer(Layer::Overlay)
                    .width(Size::Fill)
                    .height(Size::px(120.))
                    .main_align(Alignment::End)
                    .background_linear_gradient(
                        LinearGradient::new()
                            .stop((0x33000000, 20.))
                            .stop((0xb3000000, 70.)),
                    )
                    .corner_radius(28.)
                    .overflow(Overflow::Clip)
                    .child(
                        rect()
                            .horizontal()
                            .spacing(15.)
                            .cross_align(Alignment::Center)
                            .child(Avatar::new(self.user.clone(), None, 48.).presence(true))
                            .child({
                                let user = self.user.read();

                                paragraph()
                                    .font_size(14.)
                                    .span(Span::new(user.username.clone()))
                                    .span(
                                        Span::new(format!("#{}", user.discriminator))
                                            .font_weight(300),
                                    )
                            }),
                    ),
            )
    }
}

#[derive(PartialEq)]
pub struct ProfileButtons {
    pub user: Readable<v0::User>,
}

impl Component for ProfileButtons {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Users);

        let user = self.user.read();

        let (main_action, secondary_action): (
            Option<(&str, Rc<dyn Fn()>)>,
            Option<(Bytes, Rc<dyn Fn()>)>,
        ) = match &user.relationship {
            v0::RelationshipStatus::None if user.bot.is_none() => (
                Some((
                    "Add Friend",
                    Rc::new({
                        let id = user.id.clone();

                        move || {
                            let id = id.clone();

                            spawn(async move {
                                if let Ok(user) = http().add_friend_by_id(&id).await {
                                    radio.clone().write().users.insert(user.id.clone(), user);
                                }
                            });
                        }
                    }),
                )),
                None,
            ),
            v0::RelationshipStatus::Friend => (Some(("Message", Rc::new(move || {}))), None),
            v0::RelationshipStatus::Outgoing => (
                Some((
                    "Cancel Friend Request",
                    Rc::new({
                        let id = user.id.clone();
                        move || {
                            let id = id.clone();

                            spawn(async move {
                                if let Ok(user) = http().remove_friend(&id).await {
                                    radio.clone().write().users.insert(user.id.clone(), user);
                                }
                            });
                        }
                    }),
                )),
                None,
            ),
            v0::RelationshipStatus::Incoming => (
                Some((
                    "Accept Friend Request",
                    Rc::new({
                        let id = user.id.clone();
                        move || {
                            let id = id.clone();

                            spawn(async move {
                                if let Ok(user) = http().add_friend_by_id(&id).await {
                                    radio.clone().write().users.insert(user.id.clone(), user);
                                }
                            });
                        }
                    }),
                )),
                Some((
                    circle_x(),
                    Rc::new({
                        let id = user.id.clone();
                        move || {
                            let id = id.clone();

                            spawn(async move {
                                if let Ok(user) = http().remove_friend(&id).await {
                                    radio.clone().write().users.insert(user.id.clone(), user);
                                }
                            });
                        }
                    }),
                )),
            ),
            _ => (None, None),
        };

        rect()
            .horizontal()
            .width(Size::Fill)
            .main_align(Alignment::End)
            .spacing(8.)
            .maybe_child(main_action.map(|(title, callback)| {
                StoatButton::new()
                    // .key(DiffKey::new_rc(&callback))
                    .on_press(move |_| callback())
                    .child(
                        rect()
                            .horizontal()
                            .height(Size::px(40.))
                            .padding((0., 16.))
                            .center()
                            .child(
                                label()
                                    .text(title)
                                    .font_size(14.)
                                    .font_weight(FontWeight::MEDIUM),
                            ),
                    )
                    .corner_radius(40.)
                    .background(0xffb9c3ff)
                    .color(0xff202c61)
            }))
            .maybe_child(secondary_action.map(|(icon, callback)| {
                StoatButton::new()
                    // .key(DiffKey::new_rc(&callback))
                    .on_press(move |_| callback())
                    .corner_radius(40.)
                    .child(
                        rect()
                            .horizontal()
                            .height(Size::px(40.))
                            .padding((0., 8.))
                            .center()
                            .child(svg(icon).width(Size::px(24.)).height(Size::px(24.))),
                    )
            }))
            .child(
                StoatButton::new().on_press(move |_| {}).corner_radius(40.).child(
                    rect()
                        .horizontal()
                        .height(Size::px(40.))
                        .padding((0., 8.))
                        .center()
                        .child(
                            svg(ellipsis_vertical())
                                .width(Size::px(24.))
                                .height(Size::px(24.)),
                        ),
                ),
            )
    }
}

#[derive(PartialEq)]
pub struct ProfileStatus {
    pub text: String,
}

impl Component for ProfileStatus {
    fn render(&self) -> impl IntoElement {
        card("Status", label().text(self.text.clone()).font_size(14))
    }
}

#[derive(PartialEq)]
pub struct ProfileBadges {
    pub badges: u32,
}

impl Component for ProfileBadges {
    fn render(&self) -> impl IntoElement {
        let badge = |badge: v0::UserBadges, value| {
            ((self.badges & badge.clone() as u32) == badge as u32).then(|| {
                rect()
                    .center()
                    .width(Size::px(24.))
                    .height(Size::px(24.))
                    .child(value)
            })
        };

        card(
            "Badges",
            rect()
                .horizontal()
                .spacing(8.)
                .content(Content::Wrap {
                    wrap_spacing: Some(8.),
                })
                .maybe_child(badge(v0::UserBadges::Founder, "F"))
                .maybe_child(badge(v0::UserBadges::Developer, "D"))
                .maybe_child(badge(v0::UserBadges::Supporter, "S"))
                .maybe_child(badge(v0::UserBadges::Translator, "T"))
                .maybe_child(badge(v0::UserBadges::EarlyAdopter, "E"))
                .maybe_child(badge(v0::UserBadges::PlatformModeration, "M"))
                .maybe_child(badge(v0::UserBadges::ResponsibleDisclosure, "R"))
                .maybe_child(badge(v0::UserBadges::ReservedRelevantJokeBadge1, "A"))
                .maybe_child(badge(v0::UserBadges::ReservedRelevantJokeBadge2, "O"))
                .maybe_child(badge(v0::UserBadges::Paw, "P")),
        )
    }
}

#[derive(PartialEq)]
pub struct ProfileJoined {
    pub user: Readable<v0::User>,
    pub member: Option<Readable<v0::Member>>,
}

impl Component for ProfileJoined {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Servers);

        card(
            "Joined",
            rect()
                .spacing(4.)
                .child(
                    label()
                        .font_size(12.)
                        .font_weight(FontWeight::MEDIUM)
                        .text("Stoat"),
                )
                .child(label().font_size(14.).text("Aug 15 2021"))
                .map(self.member.as_ref(), |this, member| {
                    let member = member.read();
                    let state = radio.read();
                    let server = state.servers.get(&member.id.server).unwrap();

                    this.child(
                        label()
                            .font_size(12.)
                            .font_weight(FontWeight::MEDIUM)
                            .text(server.name.clone()),
                    )
                    .child(label().font_size(14.).text("Aug 15 2021"))
                }),
        )
    }
}

#[derive(PartialEq)]
pub struct ProfileBio {
    pub bio: String,
}

impl Component for ProfileBio {
    fn render(&self) -> impl IntoElement {
        card("Bio", SelectableText::new(self.bio.clone()).font_size(14)).height(Size::Inner)
    }
}

#[derive(PartialEq)]
pub struct ProfileRoles {
    pub member: Readable<v0::Member>,
}

impl Component for ProfileRoles {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Servers);

        let server = radio.slice_current({
            let member = self.member.clone();
            move |state| state.servers.get(&member.read().id.server).unwrap()
        });

        let roles = use_side_effect_value({
            let member = self.member.clone();

            move || {
                let member = member.read();
                let server = server.read();

                let mut roles = member
                    .roles
                    .iter()
                    .filter_map(|id| server.roles.get(id))
                    .map(|role| {
                        let color = role.colour.as_deref().and_then(parse_fill);

                        (role.clone(), color)
                    })
                    .collect::<Vec<_>>();

                roles.sort_by(|(a, _), (b, _)| a.rank.cmp(&b.rank));
                roles
            }
        });

        card(
            "Roles",
            ScrollView::new().children(roles.read().iter().map(|(role, color)| {
                rect()
                    .horizontal()
                    .main_align(Alignment::SpaceBetween)
                    .cross_align(Alignment::Center)
                    .width(Size::Fill)
                    .child(label().text(role.name.clone()).font_size(12.))
                    .maybe_child(color.clone().map(|color| {
                        let mut rect = rect()
                            .width(Size::px(8.))
                            .height(Size::px(8.))
                            .corner_radius(8.);

                        rect.get_style().background = color;

                        rect
                    }))
                    .into_element()
            })),
        )
    }
}
