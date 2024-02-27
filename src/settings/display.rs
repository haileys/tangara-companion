use std::fmt::Display;

use derive_more::{From, Into};

use crate::settings::{Setting, IntRangeSetting, LuaProperty, ToLuaExpr, FromLuaOutput};

#[derive(Default, From, Into, Debug)]
pub struct Brightness(pub i32);

impl Setting for Brightness {
    const PROPERTY: LuaProperty<Self> = LuaProperty::new("display", "brightness");
}

impl IntRangeSetting for Brightness {
    const MIN: i32 = 0;
    const MAX: i32 = 100;
}

impl ToLuaExpr for Brightness {
    fn to_lua_expr(&self) -> impl Display {
        self.0
    }
}

impl FromLuaOutput for Brightness {
    fn from_lua_output(output: &str) -> Option<Self> {
        output.parse().ok().map(Brightness)
    }
}
