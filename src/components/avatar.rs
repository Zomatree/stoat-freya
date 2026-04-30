use freya::prelude::*;
use stoat_models::v0;

use crate::{components::image, http};

pub fn avatar(user: &v0::User, member: Option<&v0::Member>) -> Rect {
    rect()
        .corner_radius(f32::MAX)
        .overflow(Overflow::Clip)
        .child(
            match member
                .as_ref()
                .and_then(|member| member.avatar.as_ref())
                .or_else(|| user.avatar.as_ref())
            {
                Some(file) => image(file).aspect_ratio(AspectRatio::Max).image_cover(ImageCover::Center).expanded(),
                None => ImageViewer::new(
                    http()
                        .format_default_avatar_url(&user.id)
                        .parse::<Uri>()
                        .unwrap(),
                )
                .sampling_mode(SamplingMode::Trilinear)
                .expanded(),
            },
        )
}
