use freya::prelude::*;
use stoat_models::v0;

use crate::{components::image, http};

#[derive(PartialEq)]
pub struct Avatar {
    user: Readable<v0::User>,
    member: Option<Readable<v0::Member>>,
    size: f32,
    presence: bool,
}

impl Avatar {
    pub fn new(user: Readable<v0::User>, member: Option<Readable<v0::Member>>, size: f32) -> Self {
        Self {
            user,
            member,
            size,
            presence: false,
        }
    }

    pub fn presence(mut self, presence: bool) -> Self {
        self.presence = presence;
        self
    }
}

impl Component for Avatar {
    fn render(&self) -> impl IntoElement {
        let image = self
            .member
            .as_ref()
            .and_then(|member| member.read().avatar.as_ref().map(image))
            .or_else(|| self.user.read().avatar.as_ref().map(image));

        rect()
            .width(Size::px(self.size))
            .height(Size::px(self.size))
            .corner_radius(self.size)
            .overflow(Overflow::Clip)
            .child(if let Some(image) = image {
                image
                    .aspect_ratio(AspectRatio::Max)
                    .image_cover(ImageCover::Center)
                    .expanded()
            } else {
                ImageViewer::new(
                    http()
                        .format_default_avatar_url(&self.user.peek().id)
                        .parse::<Uri>()
                        .unwrap(),
                )
                .sampling_mode(SamplingMode::Trilinear)
                .expanded()
            })
            .maybe_child(self.presence.then(|| {
                let diameter = (12. / 32.) * self.size;
                let pos = (20. / 32.) * self.size;

                let user = self.user.read();

                let presence = if !user.online {
                    v0::Presence::Invisible
                } else {
                    user.status
                    .as_ref()
                    .and_then(|status| status.presence.clone())
                    .unwrap_or(v0::Presence::Online)
                };

                let color = match presence {
                    v0::Presence::Online => 0xff3ABF7E,
                    v0::Presence::Idle => 0xffF39F00,
                    v0::Presence::Focus => 0xff4799F0,
                    v0::Presence::Busy => 0xffF84848,
                    v0::Presence::Invisible => 0xffA5A5A5,
                };

                rect()
                    .position(Position::new_absolute().left(pos).top(pos))
                    .layer(Layer::Overlay)
                    .width(Size::px(diameter))
                    .height(Size::px(diameter))
                    .corner_radius(diameter)
                    .background(color)
                    .border(Border::new().width(2.).fill(Color::BLACK))
            }))
    }

    fn render_key(&self) -> DiffKey {
        (&self.user.peek().id).into()
    }
}

// pub fn avatar(user: &v0::User, member: Option<&v0::Member>) -> Rect {

// }

// 27 / 32 = 0.84375
// 27 / 32