use freya::{icons::lucide::file_text, prelude::*};
use stoat_models::v0;

use crate::components::image;

#[derive(PartialEq)]
pub struct MessageAttachment {
    pub file: Readable<v0::File>,
}

impl Component for MessageAttachment {
    fn render(&self) -> impl IntoElement {
        rect()
            .corner_radius(12.)
            .overflow(Overflow::Clip)
            .child(match self.file.read().metadata {
                v0::Metadata::Image { width, height } => {
                    let new_width = width.min(420) as f32;
                    let new_height = (new_width / width as f32) * height as f32;

                    rect()
                        .width(Size::px(new_width))
                        .height(Size::px(new_height))
                        .child(
                            image(&self.file.read())
                                .width(Size::Fill)
                                .height(Size::Fill)
                                .aspect_ratio(AspectRatio::Min)
                                .image_cover(ImageCover::Fill),
                        )
                        .into_element()
                }
                _ => rect()
                    .padding(8.)
                    .corner_radius(12.)
                    .overflow(Overflow::Clip)
                    .width(Size::Fill)
                    .background(0xffe3e1e9)
                    .color(0xff303036)
                    .horizontal()
                    .cross_align(Alignment::Center)
                    .spacing(8.)
                    .child(svg(file_text()).width(Size::px(24.)).height(Size::px(24.)))
                    .child(
                        rect()
                            .spacing(8.)
                            .child(
                                label()
                                    .text(self.file.read().filename.clone())
                                    .font_size(14),
                            )
                            .child(
                                label()
                                    .text(format!("{} KB", self.file.read().size / 1000))
                                    .font_size(11),
                            ),
                    )
                    .into_element(),
            })
    }

    fn render_key(&self) -> DiffKey {
        (&self.file.peek().id).into()
    }
}
