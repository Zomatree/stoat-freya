use crate::use_material_theme;
use freya::prelude::*;

pub struct Dropdown<T: PartialEq + 'static, B> {
    state: Writable<T>,
    options: Vec<T>,
    builder: B,
}

impl<T: PartialEq + 'static, B> PartialEq for Dropdown<T, B> {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state && self.options == other.options
    }
}

impl<T: Clone + PartialEq + 'static, B: Fn(&T) -> Element> Dropdown<T, B> {
    pub fn new(state: impl IntoWritable<T>, options: Vec<T>, builder: B) -> Self {
        Self {
            state: state.into_writable(),
            options,
            builder,
        }
    }
}

impl<T: Clone + PartialEq + 'static, B: Fn(&T) -> Element + 'static> Component for Dropdown<T, B> {
    fn render(&self) -> impl IntoElement {
        let theme = use_material_theme();

        let current_value = &*self.state.read();

        Select::new()
            .width(Size::Fill)
            .background_button(theme.md.surface_container_highest.as_argb_u32())
            .hover_background(theme.md.surface_container_highest.as_argb_u32())
            .select_background(theme.md.surface_container.as_argb_u32())
            .selected_item(
                rect()
                    .padding((8., 6.))
                    .width(Size::Fill)
                    .child((self.builder)(current_value)),
            )
            .children(self.options.iter().map(move |value| {
                MenuItem::new()
                    .selected(value == current_value)
                    .background(theme.md.surface_container.as_argb_u32())
                    .select_background(theme.md.primary.as_u32() | (0xb9 << 24))
                    .on_press({
                        let value = value.clone();
                        let mut state = self.state.clone();

                        move |_| state.set(value.clone())
                    })
                    .child((self.builder)(value))
                    .into_element()
            }))
    }
}
