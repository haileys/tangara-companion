use derive_more::Display;

use crate::settings::{Setting, EnumSetting};

#[derive(Display, Default, PartialEq, Eq)]
pub enum InputMethod {
    #[display(fmt = "Buttons Only")]
    ButtonsOnly,
    #[display(fmt = "D-Pad")]
    DPad,
    #[display(fmt = "Touchwheel")]
    #[default]
    Touchwheel,
}

impl Setting for InputMethod {}

impl EnumSetting for InputMethod {
    const ITEMS: &'static [Self] = &[
        InputMethod::ButtonsOnly,
        InputMethod::DPad,
        InputMethod::Touchwheel,
    ];
}
