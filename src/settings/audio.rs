use std::fmt::Display;

use derive_more::{Display, From, Into};

use crate::settings::{Setting, EnumSetting, IntRangeSetting};

use super::{FromLuaOutput, LuaProperty, ToLuaExpr};

#[derive(Display, Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum MaximumVolumeLimit {
    #[display(fmt = "Line Level (-10 dB)")]
    #[default]
    Line,
    #[display(fmt = "CD Level (+6 dB)")]
    Cd,
    #[display(fmt = "Maximum (+10 dB)")]
    Maximum,
}

impl Setting for MaximumVolumeLimit {
    const PROPERTY: LuaProperty<Self> = LuaProperty::new("volume", "limit_db");
}

impl EnumSetting for MaximumVolumeLimit {
    const ITEMS: &'static [Self] = &[
        MaximumVolumeLimit::Line,
        MaximumVolumeLimit::Cd,
        MaximumVolumeLimit::Maximum,
    ];
}

impl ToLuaExpr for MaximumVolumeLimit {
    fn to_lua_expr(&self) -> impl Display {
        match self {
            MaximumVolumeLimit::Line => "1",
            MaximumVolumeLimit::Cd => "2",
            MaximumVolumeLimit::Maximum => "3",
        }
    }
}

impl FromLuaOutput for MaximumVolumeLimit {
    fn from_lua_output(output: &str) -> Option<Self> {
        match output {
            "1" => Some(MaximumVolumeLimit::Line),
            "2" => Some(MaximumVolumeLimit::Cd),
            "3" => Some(MaximumVolumeLimit::Maximum),
            _ => None,
        }
    }
}

#[derive(Default, From, Into, Debug)]
pub struct Balance(pub i32);

impl Setting for Balance {
    const PROPERTY: LuaProperty<Self> = LuaProperty::new("volume", "left_bias");
}

impl IntRangeSetting for Balance {
    const MIN: i32 = -100;
    const MAX: i32 = 100;
    const NOTCHES: &'static [(i32, Option<&'static str>)] = &[
        (-100, Some("Left")),
        (0, Some("Balanced")),
        (100, Some("Right")),
    ];
}

impl ToLuaExpr for Balance {
    fn to_lua_expr(&self) -> impl Display {
        self.0
    }
}

impl FromLuaOutput for Balance {
    fn from_lua_output(output: &str) -> Option<Self> {
        output.parse().ok().map(Balance)
    }
}

// #[derive(Default, From, Into)]
// pub struct Volume(pub i32);

// impl Setting for Volume {}

// impl IntRangeSetting for Volume {
//     const MIN: i32 = 0;
//     const MAX: i32 = 100;
// }
