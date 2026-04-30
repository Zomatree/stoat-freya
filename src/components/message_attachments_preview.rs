use std::collections::HashMap;

use freya::{
    icons::lucide::{circle_x, file_text, plus},
    prelude::*,
};
use rfd::AsyncFileDialog;

#[derive(PartialEq)]
pub struct MessageAttachmentsPreview {
    pub attachments: State<HashMap<u64, (String, Bytes)>>,
}

impl Component for MessageAttachmentsPreview {
    fn render(&self) -> impl IntoElement {
        rect()
            .horizontal()
            .padding(8.)
            .corner_radius(16.)
            .background(0xff384379)
            .width(Size::Fill)
            .child(
                ScrollView::new()
                    .height(Size::px(130.))
                    .spacing(8.)
                    .direction(Direction::Horizontal)
                    .children(self.attachments.read().keys().copied().map(|key| {
                        MessageAttachmentPreview {
                            attachments: self.attachments.clone(),
                            key,
                        }
                        .into_element()
                    }))
                    .child(
                        Button::new()
                            .color(0xffdde1ff)
                            .width(Size::px(100.))
                            .height(Size::px(100.))
                            .hover_background(0x20e3e1e9)
                            .background(Color::TRANSPARENT)
                            .border_fill(Color::TRANSPARENT)
                            .corner_radius(8.)
                            .flat()
                            .on_press({
                                let mut attachments = self.attachments.clone();

                                move |_| {
                                    spawn(async move {
                                        if let Some(file) = AsyncFileDialog::new().pick_file().await
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
                            .child(svg(plus()).width(Size::px(48.)).height(Size::px(48.))),
                    ),
            )
    }
}

#[derive(PartialEq)]
pub struct MessageAttachmentPreview {
    pub attachments: State<HashMap<u64, (String, Bytes)>>,
    pub key: u64,
}

impl Component for MessageAttachmentPreview {
    fn render(&self) -> impl IntoElement {
        let (filename, contents) = self.attachments.read().get(&self.key).unwrap().clone();

        let is_image = use_hook(|| infer::is_image(&contents));
        let mut hovering = use_state(|| false);
        let area = use_state(Area::default);

        rect()
            .cross_align(Alignment::Center)
            .color(0xffdde1ff)
            .child(
                Button::new()
                    .flat()
                    .height(Size::px(100.))
                    .background(Color::TRANSPARENT)
                    .hover_background(Color::TRANSPARENT)
                    .border_fill(Color::TRANSPARENT)
                    .padding(0.)
                    .on_press({
                        let mut attachments = self.attachments.clone();
                        let key = self.key.clone();

                        move |_| {
                            attachments.write().remove(&key);
                        }
                    })
                    .corner_radius(8.)
                    .child(
                        rect()
                            .height(Size::px(100.))
                            .on_pointer_over(move |_| {
                                hovering.set(true);
                            })
                            .on_pointer_out(move |_| hovering.set_if_modified(false))
                            .child(
                                rect()
                                    .child(if is_image {
                                        ImageViewer::new(ImageSource::Bytes(
                                            self.key,
                                            contents.clone(),
                                        ))
                                        .into_element()
                                    } else {
                                        rect()
                                            .width(Size::px(100.))
                                            .height(Size::px(100.))
                                            .center()
                                            .child(
                                                svg(file_text())
                                                    .width(Size::px(36.))
                                                    .height(Size::px(36.)),
                                            )
                                            .into_element()
                                    })
                                    .on_sized({
                                        let mut area = area.clone();
                                        move |e: Event<SizedEventData>| area.set(e.area)
                                    }),
                            )
                            .maybe_child(hovering.read().then(|| {
                                rect()
                                    .position(Position::new_absolute())
                                    .width(Size::px(area.read().width()))
                                    .height(Size::px(100.))
                                    .layer(Layer::RelativeOverlay(1))
                                    .background(0xcc000000)
                                    .center()
                                    .child(
                                        svg(circle_x())
                                            .width(Size::px(36.))
                                            .height(Size::px(36.))
                                            .color(Color::WHITE),
                                    )
                            })),
                    ),
            )
            .child(
                label()
                    .text(filename)
                    .max_lines(1)
                    .text_overflow(TextOverflow::Ellipsis)
                    .font_size(12)
                    .max_width(Size::px(100.)),
            )
            .child(
                label()
                    .text(format!("{} KB", contents.len() / 1000))
                    .font_size(11),
            )
    }

    fn render_key(&self) -> DiffKey {
        (&self.key).into()
    }
}
