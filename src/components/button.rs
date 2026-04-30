use freya::prelude::{
    AccessibilityExt, AccessibilityRole, ChildrenExt, Color, Component, ContainerExt,
    ContainerSizeExt, CornerRadius, CornerRadiusExt, Cursor, CursorIcon, Element, Event,
    EventHandler, EventHandlersExt, Gaps, IntoElement, MouseButton, Overflow, PressEventData, Size,
    StyleExt, WritableUtils, define_theme, get_theme, rect, use_drop, use_focus, use_state,
};

define_theme! {
    for = StoatButton;
    theme_field = theme_layout;

    %[component]
    pub StoatButtonLayout {
        %[fields]
        margin: Gaps,
        corner_radius: CornerRadius,
        width: Size,
        height: Size,
        padding: Gaps,
    }
}

define_theme! {
    for = StoatButton;
    theme_field = theme_colors;

    %[component]
    pub StoatButtonColors {
        %[fields]
        background: Color,
        hover_background: Color,
        border_fill: Color,
        focus_border_fill: Color,
        color: Color,
    }
}

#[derive(PartialEq)]
pub struct StoatButton {
    theme_colors: Option<StoatButtonColorsThemePartial>,
    theme_layout: Option<StoatButtonLayoutThemePartial>,
    elements: Vec<Element>,
    on_press: Option<EventHandler<Event<PressEventData>>>,
}

impl StoatButton {
    pub fn new() -> Self {
        Self {
            theme_colors: None,
            theme_layout: None,
            on_press: None,
            elements: Vec::default(),
        }
    }

    pub fn on_press(mut self, on_press: impl Into<EventHandler<Event<PressEventData>>>) -> Self {
        self.on_press = Some(on_press.into());
        self
    }
}

impl ChildrenExt for StoatButton {
    fn get_children(&mut self) -> &mut Vec<Element> {
        &mut self.elements
    }
}

impl CornerRadiusExt for StoatButton {
    fn with_corner_radius(self, corner_radius: f32) -> Self {
        self.corner_radius(corner_radius)
    }
}

impl Component for StoatButton {
    fn render(&self) -> impl IntoElement {
        let mut hovering = use_state(|| false);
        let focus = use_focus();

        use_drop(move || {
            if hovering() {
                Cursor::set(CursorIcon::default());
            }
        });

        let theme_colors = get_theme!(
            &self.theme_colors,
            StoatButtonColorsThemePreference,
            "stoat_button"
        );
        let theme_layout = get_theme!(
            &self.theme_layout,
            StoatButtonLayoutThemePreference,
            "stoat_button_layout"
        );

        let background = if hovering() && theme_colors.hover_background != Color::TRANSPARENT {
            theme_colors.hover_background
        } else {
            theme_colors.background
        };

        rect()
            .overflow(Overflow::Clip)
            .a11y_id(focus.a11y_id())
            .a11y_role(AccessibilityRole::Button)
            .background(background)
            .padding(theme_layout.padding)
            .corner_radius(theme_layout.corner_radius)
            .width(theme_layout.width)
            .height(theme_layout.height)
            .color(theme_colors.color)
            .on_all_press({
                let on_press = self.on_press.clone();
                move |e: Event<PressEventData>| {
                    focus.request_focus();
                    match e.data() {
                        PressEventData::Mouse(data) => match data.button {
                            Some(MouseButton::Left) => {
                                if let Some(handler) = &on_press {
                                    handler.call(e);
                                }
                            }
                            _ => {}
                        },
                        PressEventData::Touch(_) | PressEventData::Keyboard(_) => {
                            if let Some(handler) = &on_press {
                                handler.call(e);
                            }
                        }
                    }
                }
            })
            .on_pointer_over(move |_| {
                hovering.set(true);
            })
            .on_pointer_out(move |_| hovering.set_if_modified(false))
            .on_pointer_enter(move |_| {
                Cursor::set(CursorIcon::Pointer);
            })
            .on_pointer_leave(move |_| {
                Cursor::set(CursorIcon::default());
            })
            .children(self.elements.clone())
    }
}
