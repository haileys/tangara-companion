use derive_more::{From, Into};

use crate::settings::{Setting, IntRangeSetting};

#[derive(Default, From, Into)]
pub struct Brightness(pub i32);

impl Setting for Brightness {}

impl IntRangeSetting for Brightness {
    const MIN: i32 = 0;
    const MAX: i32 = 100;
}
