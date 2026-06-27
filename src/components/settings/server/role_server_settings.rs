use freya::{
    icons::lucide::{chevron_right, list, user_plus},
    prelude::*,
    radio::use_radio,
};
use stoat_models::v0;

use crate::{
    AppChannel, ServerSettingsPage,
    components::{ModalValue, StoatButton, StoatButtonLayoutThemePartialExt, use_modals},
    parse_fill, use_material_theme,
};

#[derive(Debug, Clone, PartialEq)]
pub enum SelectedRole {
    Default,
    Role(String),
}

#[derive(PartialEq)]
pub struct RoleServerSettings {
    pub server: Readable<v0::Server>,
    pub selected_role: Option<SelectedRole>,
}

impl Component for RoleServerSettings {
    fn render(&self) -> impl IntoElement {
        let theme = use_material_theme();
        let mut modals = use_modals();
        let mut radio = use_radio(AppChannel::ServerSettingsPage);

        let set_selected_role = {
            let server = self.server.read().id.clone();
            move |role| {
                radio.write().server_settings_page =
                    Some((server.clone(), ServerSettingsPage::Roles(Some(role))))
            }
        };

        let ordered_roles = use_memo({
            let server = self.server.clone();
            move || {
                let mut roles = server
                    .read()
                    .roles
                    .values()
                    .cloned()
                    .map(|role| {
                        let color = role.colour.as_deref().and_then(parse_fill);
                        (role, color)
                    })
                    .collect::<Vec<_>>();

                roles.sort_by(|(a, _), (b, _)| a.rank.cmp(&b.rank));
                roles
            }
        });

        match self.selected_role.clone() {
            Some(SelectedRole::Default) => "defaut".into_element(),
            Some(SelectedRole::Role(id)) => id.into_element(),
            None => rect()
                .spacing(15.)
                .child(
                    rect()
                        .spacing(4.)
                        .child(
                            StoatButton::new()
                                .corner_radius(12.)
                                .on_press({
                                    let mut set_selected_role = set_selected_role.clone();
                                    move |_| set_selected_role(SelectedRole::Default)
                                })
                                .child(
                                    rect()
                                        .padding(13.)
                                        .background(theme.md.secondary_container.as_argb_u32())
                                        .color(theme.md.on_secondary_container.as_argb_u32())
                                        .child(
                                            rect()
                                                .horizontal()
                                                .spacing(16.)
                                                .cross_align(Alignment::Center)
                                                .content(Content::Flex)
                                                .child(
                                                    rect()
                                                        .corner_radius(36.)
                                                        .width(Size::px(36.))
                                                        .height(Size::px(36.))
                                                        .background(
                                                            theme.md.surface_dim.as_argb_u32(),
                                                        )
                                                        .color(theme.md.on_surface.as_argb_u32())
                                                        .center()
                                                        .child(
                                                            svg(list())
                                                                .width(Size::px(22.))
                                                                .height(Size::px(22.)),
                                                        ),
                                                )
                                                .child(
                                                    rect()
                                                        .width(Size::flex(1.))
                                                        .child(
                                                            label()
                                                                .font_size(14.)
                                                                .font_weight(FontWeight::SEMI_BOLD)
                                                                .line_height(1.5)
                                                                .text("Default Permissions"),
                                                        )
                                                        .child(
                                                            label()
                                                                .font_size(12.)
                                                                .line_height(1.5)
                                                                .text(
                                                                    "Affects all roles and users",
                                                                ),
                                                        ),
                                                )
                                                .child(
                                                    svg(chevron_right())
                                                        .width(Size::px(18.))
                                                        .height(Size::px(18.)),
                                                ),
                                        ),
                                ),
                        )
                        .child(
                            StoatButton::new()
                                .corner_radius(12.)
                                .on_press({
                                    let id = self.server.read().id.clone();
                                    move |_| {
                                        modals.write().push_modal(ModalValue::CreateRole {
                                            server: id.clone(),
                                        })
                                    }
                                })
                                .child(
                                    rect()
                                        .padding(13.)
                                        .background(theme.md.secondary_container.as_argb_u32())
                                        .color(theme.md.on_secondary_container.as_argb_u32())
                                        .child(
                                            rect()
                                                .horizontal()
                                                .spacing(16.)
                                                .cross_align(Alignment::Center)
                                                .content(Content::Flex)
                                                .child(
                                                    rect()
                                                        .corner_radius(36.)
                                                        .width(Size::px(36.))
                                                        .height(Size::px(36.))
                                                        .background(
                                                            theme.md.surface_dim.as_argb_u32(),
                                                        )
                                                        .color(theme.md.on_surface.as_argb_u32())
                                                        .center()
                                                        .child(
                                                            svg(user_plus())
                                                                .width(Size::px(22.))
                                                                .height(Size::px(22.)),
                                                        ),
                                                )
                                                .child(
                                                    rect()
                                                        .width(Size::flex(1.))
                                                        .child(
                                                            label()
                                                                .font_size(14.)
                                                                .font_weight(FontWeight::SEMI_BOLD)
                                                                .line_height(1.5)
                                                                .text("Create Role"),
                                                        )
                                                        .child(
                                                            label()
                                                                .font_size(12.)
                                                                .line_height(1.5)
                                                                .text("Create a new role"),
                                                        ),
                                                )
                                                .child(
                                                    svg(chevron_right())
                                                        .width(Size::px(18.))
                                                        .height(Size::px(18.)),
                                                ),
                                        ),
                                ),
                        ),
                )
                .child(
                    rect()
                        .spacing(4.)
                        .child(label().font_size(12.).text("Server Roles"))
                        .child(rect().spacing(8.).children(ordered_roles.read().iter().map(
                            |(role, color)| {
                                let mut role_color =
                                    rect().background(theme.md.outline_variant.as_argb_u32());

                                if let Some(color) = color {
                                    role_color.get_style().background = color.clone();
                                };

                                StoatButton::new()
                                    .corner_radius(12.)
                                    .on_press({
                                        let id = role.id.clone();
                                        let mut set_selected_role = set_selected_role.clone();

                                        move |_| set_selected_role(SelectedRole::Role(id.clone()))
                                    })
                                    .child(
                                        rect()
                                            .padding(13.)
                                            .background(theme.md.secondary_container.as_argb_u32())
                                            .color(theme.md.on_secondary_container.as_argb_u32())
                                            .child(
                                                rect()
                                                    .horizontal()
                                                    .spacing(16.)
                                                    .cross_align(Alignment::Center)
                                                    .content(Content::Flex)
                                                    .child(
                                                        role_color
                                                            .corner_radius(36.)
                                                            .width(Size::px(36.))
                                                            .height(Size::px(36.)),
                                                    )
                                                    .child(
                                                        label()
                                                            .width(Size::flex(1.))
                                                            .font_size(14.)
                                                            .font_weight(FontWeight::SEMI_BOLD)
                                                            .line_height(1.5)
                                                            .text(role.name.clone()),
                                                    )
                                                    .child(
                                                        svg(chevron_right())
                                                            .width(Size::px(18.))
                                                            .height(Size::px(18.)),
                                                    ),
                                            ),
                                    )
                                    .into_element()
                            },
                        ))),
                )
                .into_element(),
        }
    }
}
