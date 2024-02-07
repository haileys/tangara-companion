use std::fmt::Display;

pub mod display;
pub mod audio;
pub mod input;

pub trait Setting: Sized + Default + 'static {
    // type Error;
    // async fn get(conn: &Connection) -> Result<Self, Self::Error>;
    // async fn set(conn: &Connection, value: &Self) -> Result<(), Self::Error>;
}

pub trait EnumSetting: Setting + Display + Eq {
    const ITEMS: &'static [Self];
}

pub trait IntRangeSetting: Setting + From<i32> + Into<i32> {
    const MIN: i32;
    const MAX: i32;
    const NOTCHES: &'static [(i32, Option<&'static str>)] = &[];
}
