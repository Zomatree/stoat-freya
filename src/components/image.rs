use freya::prelude::*;
use stoat_models::v0;

use crate::http;

pub fn image(file: &v0::File) -> ImageViewer {
    ImageViewer::new(
        format!(
            "{}/{}/{}",
            http().api_config.features.autumn.url,
            &file.tag,
            &file.id
        )
        .parse::<Uri>()
        .unwrap(),
    )
    .sampling_mode(SamplingMode::Trilinear)
}
