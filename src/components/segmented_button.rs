use std::{fmt::Debug, hash::Hash};

use freya::{
    animation::{
        AnimNum, AnimatedValue, Ease, OnChange, OnCreation, use_animation_with_dependencies
    },
    prelude::*,
};

use crate::{
    components::{StoatButton, StoatButtonLayoutThemePartialExt}, use_material_theme
};

pub struct StoatSegmentedButton<T: PartialEq + 'static, B> {
    state: Writable<T>,
    options: Vec<T>,
    builder: B,
    height: Size,
}

impl<T: PartialEq + 'static, B> PartialEq for StoatSegmentedButton<T, B> {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state && self.options == other.options && self.height == other.height
    }
}

impl<T: Clone + PartialEq + 'static, B: Fn(&T) -> Element> StoatSegmentedButton<T, B> {
    pub fn new(state: impl IntoWritable<T>, options: Vec<T>, builder: B) -> Self {
        Self {
            state: state.into_writable(),
            options,
            builder,
            height: Size::Fill,
        }
    }

    pub fn height(mut self, size: impl Into<Size>) -> Self {
        self.height = size.into();

        self
    }
}

impl<T: Debug + Hash + Clone + PartialEq + 'static, B: Fn(&T) -> Element + 'static> Component
    for StoatSegmentedButton<T, B>
{
    fn render(&self) -> impl IntoElement {
        let children = self
            .options
            .iter()
            .enumerate()
            .map(|(i, value)| (i, value, (self.builder)(value)));

        rect()
            .horizontal()
            .content(Content::Flex)
            .spacing(8.)
            .children(children.map(|(i, value, element)| {
                StoatInnerSegmentedButton {
                    state: self.state.clone(),
                    value: value.clone(),
                    height: self.height.clone(),
                    child: element,
                    first: i == 0,
                    last: i == self.options.len() - 1,
                }
                .into_element()
            }))
    }
}

#[derive(PartialEq)]
struct StoatInnerSegmentedButton<T: 'static> {
    pub state: Writable<T>,
    pub value: T,
    pub height: Size,
    pub child: Element,
    pub first: bool,
    pub last: bool,
}

impl<T: Hash + Clone + PartialEq + 'static> Component for StoatInnerSegmentedButton<T> {
    fn render(&self) -> impl IntoElement {
        const ANIM_TIME: u64 = 250;

        // let theme = consume_context::<State<Theme>>();
        let theme = use_material_theme();

        // let selected = use_reactive(&(*self.selected.read() == Some(self.idx)));

        // let corners_animation = use_animation({let first = self.first.clone(); let last = self.last.clone(); move |conf| {
        //     conf.on_change(OnChange::Rerun);
        //     conf.on_creation(OnCreation::Finish);

        //     let selected = *selected.read();

        //     let anim = |a, b| AnimNum::new(a, b).time(ANIM_TIME).ease(Ease::InOut);

        //     let left = if first {
        //         anim(48., 48.)
        //     } else {
        //         anim(12., 48.)
        //     };
        //     // let right = if last {
        //     //     let mut a = anim(48., 48.);
        //     //     a.finish(AnimDirection::Forward);
        //     //     a
        //     // } else {
        //     //     anim(12., 48.)
        //     // };

        //     // let left = anim(12., 48.);
        //     let right = anim(12., 48.);

        //     if selected {
        //         (left, right)
        //     } else {
        //         (
        //             left.into_reversed(),
        //             right.into_reversed(),
        //         )
        //     }
        // }});

        // let top_left_corners_animation = use_animation({let first = self.first.clone(); let last = self.last.clone(); move |conf| {
        //     conf.on_change(OnChange::Rerun);
        //     conf.on_creation(OnCreation::Finish);

        //     let selected = *selected.read();

        //     let anim = if first {
        //         AnimNum::new(48., 48.).time(ANIM_TIME).ease(Ease::InOut)
        //     } else {
        //         AnimNum::new(12., 48.).time(ANIM_TIME).ease(Ease::InOut)
        //     };

        //     if selected {
        //         anim
        //     } else {
        //         anim.into_reversed()
        //     }
        // }});

        // let top_right_corners_animation = use_animation({let first = self.first.clone(); let last = self.last.clone(); move |conf| {
        //     conf.on_change(OnChange::Rerun);
        //     conf.on_creation(OnCreation::Finish);

        //     let selected = *selected.read();

        //     let anim = if last {
        //         AnimNum::new(48., 48.).time(ANIM_TIME).ease(Ease::InOut)
        //     } else {
        //         AnimNum::new(12., 48.).time(ANIM_TIME).ease(Ease::InOut)
        //     };

        //     if selected {
        //         anim
        //     } else {
        //         anim.into_reversed()
        //     }
        // }});

        let selected = &*self.state.read() == &self.value;

        let corners_animation = use_animation_with_dependencies(&selected, {move |conf, selected| {
            conf.on_change(OnChange::Rerun);
            conf.on_creation(OnCreation::Finish);

            let anim = AnimNum::new(12., 48.).time(ANIM_TIME).ease(Ease::InOut);

            if *selected {
                anim
            } else {
                anim.into_reversed()
            }
        }});

        // use_animation_with_dependencies(dependencies, run)

        // let top_right_corners_animation = use_animation({let first = self.first.clone(); let last = self.last.clone(); move |conf| {
        //     conf.on_change(OnChange::Rerun);
        //     conf.on_creation(OnCreation::Finish);

        //     let selected = *selected.read();

        //     let anim = if last {
        //         AnimNum::new(48., 48.).time(ANIM_TIME).ease(Ease::InOut)
        //     } else {
        //         AnimNum::new(12., 48.).time(ANIM_TIME).ease(Ease::InOut)
        //     };

        //     if selected {
        //         anim
        //     } else {
        //         anim.into_reversed()
        //     }
        // }});

        // let colors_animation = use_animation({move |conf| {
        //     const ANIM_TIME: u64 = 250;

        //     conf.on_change(OnChange::Rerun);
        //     conf.on_creation(OnCreation::Finish);

        //     let selected = *selected.read();
        //     let theme = theme.read();

        //     let color = AnimColor::new(theme.md.on_secondary_container.as_argb_u32(), theme.md.on_primary.as_argb_u32()).time(ANIM_TIME);
        //     let background = AnimColor::new(theme.md.secondary_container.as_argb_u32(), theme.md.primary.as_argb_u32()).time(ANIM_TIME);

        //     if selected {
        //         (color, background)
        //     } else {
        //         (
        //             color.into_reversed(),
        //             background.into_reversed(),
        //         )
        //     }
        // }});

        // let animation = use_animation_transition((self.selected, self.first, self.last), |_, (selected, first, last)| {

        // });

        // let corners = if self.selected {
        //     CornerRadius {
        //         top_left: 48.,
        //         top_right: 48.,
        //         bottom_right: 48.,
        //         bottom_left: 48.,
        //         smoothing: 0.,
        //     }
        // } else if self.first {
        //     CornerRadius {
        //         top_left: 48.,
        //         top_right: 12.,
        //         bottom_right: 12.,
        //         bottom_left: 48.,
        //         smoothing: 0.,
        //     }
        // } else if self.last {
        //     CornerRadius {
        //         top_left: 12.,
        //         top_right: 48.,
        //         bottom_right: 48.,
        //         bottom_left: 12.,
        //         smoothing: 0.,
        //     }
        // } else {
        //     CornerRadius {
        //         top_left: 12.,
        //         top_right: 12.,
        //         bottom_right: 12.,
        //         bottom_left: 12.,
        //         smoothing: 0.,
        //     }
        // };

        // let (left, right) = {
        //     let r = corners_animation.read();

        //     (r.0.value(), r.1.value())
        // };

        let corners = CornerRadius {
            top_left: if self.first { 48. } else { corners_animation.read().value() },
            top_right: if self.last { 48. } else { corners_animation.read().value() },
            bottom_right: if self.last { 48. } else { corners_animation.read().value() },
            bottom_left: if self.first { 48. } else { corners_animation.read().value() },
            smoothing: 1.,
        };

        // let (color, background) = {
        //     let r = colors_animation.read();

        //     (r.0.value(), r.1.value())
        // };

        let (color, background) = if selected {
            (theme.md.on_primary, theme.md.primary)
        } else {
            (
                theme.md.on_secondary_container,
                theme.md.secondary_container,
            )
        };

        StoatButton::new()
            .width(Size::flex(1.))
            .corner_radius(corners)
            .child(
                rect()
                    .width(Size::Fill)
                    .color(color.as_argb_u32())
                    .background(background.as_argb_u32())
                    .height(self.height.clone())
                    .center()
                    .child(self.child.clone()),
            )
            .on_press({
                let state = self.state.clone();
                let value = self.value.clone();

                move |_| state.clone().set(value.clone())
            })
    }

    fn render_key(&self) -> DiffKey {
        (&self.value).into()
    }
}
