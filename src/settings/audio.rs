use derive_more::{Display, From, Into};

use crate::settings::{Setting, EnumSetting, IntRangeSetting};

#[derive(Display, Default, PartialEq, Eq)]
pub enum MaximumVolumeLimit {
    #[display(fmt = "Line Level (-10 dB)")]
    #[default]
    Line,
    #[display(fmt = "CD Level (+6 dB)")]
    Cd,
    #[display(fmt = "Maximum (+10 dB)")]
    Maximum,
    #[display(fmt = "{:+} dB", "_0")]
    #[allow(unused)]
    Custom(i32),
}

impl Setting for MaximumVolumeLimit {}

impl EnumSetting for MaximumVolumeLimit {
    const ITEMS: &'static [Self] = &[
        MaximumVolumeLimit::Line,
        MaximumVolumeLimit::Cd,
        MaximumVolumeLimit::Maximum,
    ];
}

#[derive(Default, From, Into)]
pub struct Balance(pub i32);

impl Setting for Balance {}

impl IntRangeSetting for Balance {
    const MIN: i32 = -100;
    const MAX: i32 = 100;
    const NOTCHES: &'static [(i32, Option<&'static str>)] = &[
        (-100, Some("Left")),
        (0, Some("Balanced")),
        (100, Some("Right")),
    ];
}

#[derive(Default, From, Into)]
pub struct Volume(pub i32);

impl Setting for Volume {}

impl IntRangeSetting for Volume {
    const MIN: i32 = 0;
    const MAX: i32 = 100;
}
