use freya::{prelude::*, radio::use_radio};

use crate::{AppChannel, ServerSettingsPage, components::Dialog, use_material_theme};

#[derive(PartialEq)]
pub struct ServerInfoModal {
    pub server: String,
}

impl Component for ServerInfoModal {
    fn render(&self) -> impl IntoElement {
        let theme = use_material_theme();
        let radio = use_radio(AppChannel::Servers);

        let server = radio.read().servers.get(&self.server).unwrap().clone();
        let server_settings = radio.slice_mut(AppChannel::ServerSettingsPage, |state| {
            &mut state.server_settings_page
        });

        Dialog::new()
            .title(label().line_height(1.5).text(server.name))
            .body(rect().maybe_child(server.description.map(|description| {
                label()
                    .color(theme.md.on_surface_variant.as_argb_u32())
                    .font_weight(400)
                    .font_size(14.)
                    .text(description)
            })))
            .action("Settings", move || {
                *server_settings.clone().write() =
                    Some((server.id.clone(), ServerSettingsPage::default()));
                true
            })
            .default_action("Close")
    }
}
