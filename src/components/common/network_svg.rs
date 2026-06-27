use std::rc::Rc;

use freya::prelude::*;
use freya_components::cache::{Asset, AssetAge, AssetCacher, AssetConfiguration, use_asset};

#[derive(PartialEq)]
pub struct NetworkSvg {
    url: Uri,
    layout: LayoutData,
    accessibility: AccessibilityData,
    loading_placeholder: Option<Element>,
    key: DiffKey,
}

impl NetworkSvg {
    pub fn new(url: impl Into<Uri>) -> Self {
        Self {
            url: url.into(),
            layout: LayoutData::default(),
            accessibility: AccessibilityData::default(),
            loading_placeholder: None,
            key: DiffKey::None,
        }
    }
}

impl KeyExt for NetworkSvg {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl LayoutExt for NetworkSvg {
    fn get_layout(&mut self) -> &mut LayoutData {
        &mut self.layout
    }
}

impl ContainerSizeExt for NetworkSvg {}

impl AccessibilityExt for NetworkSvg {
    fn get_accessibility_data(&mut self) -> &mut AccessibilityData {
        &mut self.accessibility
    }
}

impl Component for NetworkSvg {
    fn render(&self) -> impl IntoElement {
        let asset_config = AssetConfiguration::new(&self.url, AssetAge::default());
        let asset = use_asset(&asset_config);
        let mut asset_cacher = use_hook(AssetCacher::get);

        use_side_effect_with_deps(
            &(self.url.clone(), asset_config),
            move |(url, asset_config): &(Uri, AssetConfiguration)| {
                // Fetch asset if still pending or errored. The Loading state
                // guards against duplicate in-flight fetches.
                if matches!(
                    asset_cacher.read_asset(asset_config),
                    Some(Asset::Pending) | Some(Asset::Error(_))
                ) {
                    asset_cacher.update_asset(asset_config.clone(), Asset::Loading);

                    let url = url.clone();
                    let asset_config = asset_config.clone();
                    spawn_forever(async move {
                        let url = url.clone();

                        let bytes = blocking::unblock::<Result<Bytes, ureq::Error>, _>(move || {
                            Ok(ureq::get(url)
                                .call()?
                                .body_mut()
                                .read_to_vec()
                                .map(Bytes::from)?)
                        })
                        .await;

                        match bytes {
                            Ok(bytes) => {
                                asset_cacher
                                    .update_asset(asset_config, Asset::Cached(Rc::new(bytes)));
                            }
                            Err(err) => {
                                // Image errored
                                asset_cacher
                                    .update_asset(asset_config, Asset::Error(err.to_string()));
                            }
                        }
                    });
                }
            },
        );

        match asset {
            Asset::Cached(asset) => {
                let asset = asset.downcast_ref::<Bytes>().unwrap().clone();
                svg(asset)
                    .accessibility(self.accessibility.clone())
                    .a11y_role(AccessibilityRole::SvgRoot)
                    .a11y_focusable(true)
                    .layout(self.layout.clone())
                    .into_element()
            }
            Asset::Pending | Asset::Loading => rect()
                .layout(self.layout.clone())
                .center()
                .child(
                    self.loading_placeholder
                        .clone()
                        .unwrap_or_else(|| CircularLoader::new().into_element()),
                )
                .into(),
            Asset::Error(err) => err.into(),
        }
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}
