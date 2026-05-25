use freya::prelude::Color;
use material_colors::scheme::Scheme;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    pub md: Scheme,
    pub stoat: StoatScheme,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StoatScheme {
    pub presence_online: Color,
    pub presence_idle: Color,
    pub presence_busy: Color,
    pub presence_focus: Color,
    pub presence_invisible: Color,
}

impl Default for StoatScheme {
    fn default() -> Self {
        Self {
            presence_online: 0xff3ABF7E.into(),
            presence_idle: 0xffF39F00.into(),
            presence_busy: 0xffF84848.into(),
            presence_focus: 0xff4799F0.into(),
            presence_invisible: 0xffA5A5A5.into(),
        }
    }
}