use crate::{
    LocalFile,
    components::{StoatButton, StoatButtonLayoutThemePartialExt, image},
    http,
    types::Tag,
    use_material_theme,
};
use freya::{icons::lucide::x, prelude::*};
use rfd::AsyncFileDialog;
use stoat_models::v0;

#[derive(PartialEq)]
pub struct OverviewServerSettings {
    pub server: Readable<v0::Server>,
}

impl Component for OverviewServerSettings {
    fn render(&self) -> impl IntoElement {
        let theme = use_material_theme();

        let edit_server = {
            let server_id = self.server.read().id.clone();

            move |payload| {
                let server_id = server_id.clone();
                async move { if let Ok(server) = http().edit_server(&server_id, &payload).await {} }
            }
        };

        let remove_field = {
            let edit_server = edit_server.clone();

            move |field| {
                let edit_server = edit_server.clone();

                async move {
                    edit_server(v0::DataEditServer {
                        name: None,
                        description: None,
                        icon: None,
                        banner: None,
                        categories: None,
                        system_messages: None,
                        flags: None,
                        discoverable: None,
                        analytics: None,
                        owner: None,
                        remove: vec![field],
                    })
                    .await
                }
            }
        };

        let prompt_image_upload = {
            move |tag: Tag| async move {
                if let Some(file) = AsyncFileDialog::new().pick_file().await {
                    let contents = file.read().await.into();
                    let filename = file.file_name();

                    if let Ok(response) = http()
                        .upload_file(
                            tag.as_str(),
                            LocalFile {
                                name: filename,
                                body: contents,
                            },
                        )
                        .await
                    {
                        return Some(response.id);
                    };
                };

                None
            }
        };

        rect()
            .spacing(8.)
            .child(label().text("Server Icon").font_size(12.))
            .child(
                rect()
                    .horizontal()
                    .spacing(15.)
                    .cross_align(Alignment::Center)
                    .child(
                        StoatButton::new()
                            .corner_radius(48.)
                            .on_press({
                                let prompt_image_upload = prompt_image_upload.clone();
                                let edit_server = edit_server.clone();

                                move |_| {
                                    let prompt_image_upload = prompt_image_upload.clone();
                                    let edit_server = edit_server.clone();

                                    spawn(async move {
                                        if let Some(id) = prompt_image_upload(Tag::Icons).await {
                                            edit_server(v0::DataEditServer {
                                                name: None,
                                                description: None,
                                                icon: Some(id),
                                                banner: None,
                                                categories: None,
                                                system_messages: None,
                                                flags: None,
                                                discoverable: None,
                                                analytics: None,
                                                owner: None,
                                                remove: Vec::new(),
                                            })
                                            .await;
                                        };
                                    });
                                }
                            })
                            .child(
                                rect()
                                    .width(Size::px(96.))
                                    .height(Size::px(96.))
                                    .background(theme.md.surface_dim.as_argb_u32())
                                    .maybe_child(self.server.read().icon.as_ref().map(|icon| {
                                        rect()
                                            .layer(Layer::Relative(1))
                                            .width(Size::Fill)
                                            .height(Size::Fill)
                                            .child(image(icon))
                                    })),
                            ),
                    )
                    .child(
                        StoatButton::new()
                            .corner_radius(16.)
                            .on_press({
                                let remove_field = remove_field.clone();

                                move |_| {
                                    let remove_field = remove_field.clone();

                                    spawn(
                                        async move { remove_field(v0::FieldsServer::Icon).await },
                                    );
                                }
                            })
                            .child(
                                rect().padding(4.).child(
                                    svg(x())
                                        .width(Size::px(24.))
                                        .height(Size::px(24.))
                                        .color(theme.md.primary.as_argb_u32()),
                                ),
                            ),
                    ),
            )
            .child(label().text("Server Banner").font_size(12.))
            .child(
                rect()
                    .horizontal()
                    .spacing(15.)
                    .cross_align(Alignment::Center)
                    .child(
                        StoatButton::new()
                            .corner_radius(16.)
                            .on_press({
                                let prompt_image_upload = prompt_image_upload.clone();
                                let edit_server = edit_server.clone();

                                move |_| {
                                    let prompt_image_upload = prompt_image_upload.clone();
                                    let edit_server = edit_server.clone();

                                    spawn(async move {
                                        if let Some(id) = prompt_image_upload(Tag::Banners).await {
                                            edit_server(v0::DataEditServer {
                                                name: None,
                                                description: None,
                                                icon: None,
                                                banner: Some(id),
                                                categories: None,
                                                system_messages: None,
                                                flags: None,
                                                discoverable: None,
                                                analytics: None,
                                                owner: None,
                                                remove: Vec::new(),
                                            })
                                            .await;
                                        };
                                    });
                                }
                            })
                            .child(
                                rect()
                                    .width(Size::px((96. / 100.) * 232.))
                                    .height(Size::px(96.))
                                    .background(theme.md.surface_dim.as_argb_u32())
                                    .maybe_child(self.server.read().banner.as_ref().map(|icon| {
                                        rect()
                                            .layer(Layer::Relative(1))
                                            .width(Size::Fill)
                                            .height(Size::Fill)
                                            .child(image(icon).aspect_ratio(AspectRatio::Max))
                                    })),
                            ),
                    )
                    .child(
                        StoatButton::new()
                            .corner_radius(16.)
                            .on_press({
                                let remove_field = remove_field.clone();

                                move |_| {
                                    let remove_field = remove_field.clone();

                                    spawn(
                                        async move { remove_field(v0::FieldsServer::Banner).await },
                                    );
                                }
                            })
                            .child(
                                rect().padding(4.).child(
                                    svg(x())
                                        .width(Size::px(24.))
                                        .height(Size::px(24.))
                                        .color(theme.md.primary.as_argb_u32()),
                                ),
                            ),
                    ),
            )
    }
}
