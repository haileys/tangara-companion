use std::fmt::Display;

use derive_more::Display;
use tangara_lib::device::{Connection, LuaError};

use crate::settings::{Setting, EnumSetting};

use super::{FromLuaOutput, LuaProperty, ToLuaExpr};

#[derive(Display, Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum InputMethod {
    #[display(fmt = "Buttons Only")]
    ButtonsOnly,
    #[display(fmt = "D-Pad")]
    DPad,
    #[display(fmt = "Touchwheel")]
    #[default]
    Touchwheel,
}

impl Setting for InputMethod {
    const PROPERTY: super::LuaProperty<Self> = LuaProperty::new("controls", "scheme");
}

impl EnumSetting for InputMethod {
    const ITEMS: &'static [Self] = &[
        InputMethod::ButtonsOnly,
        InputMethod::DPad,
        InputMethod::Touchwheel,
    ];
}

impl ToLuaExpr for InputMethod {
    fn to_lua_expr(&self) -> impl Display {
        match self {
            InputMethod::ButtonsOnly => "0",
            InputMethod::DPad => "2",
            InputMethod::Touchwheel => "3",
        }
    }
}

impl FromLuaOutput for InputMethod {
    fn from_lua_output(output: &str) -> Option<Self> {
        match output {
            "0" => Some(InputMethod::ButtonsOnly),
            "2" => Some(InputMethod::DPad),
            "3" => Some(InputMethod::Touchwheel),
            _ => None,
        }
    }
}
