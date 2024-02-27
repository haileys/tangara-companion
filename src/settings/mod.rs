use std::{fmt::Display, marker::PhantomData};

use tangara_lib::device::{LuaError, Connection};

pub mod display;
pub mod audio;
pub mod input;

pub trait Setting: Sized + Default + ToLuaExpr + FromLuaOutput + 'static {
    const PROPERTY: LuaProperty<Self>;
}

pub trait EnumSetting: Setting + Display + Eq {
    const ITEMS: &'static [Self];
}

pub trait IntRangeSetting: Setting + From<i32> + Into<i32> {
    const MIN: i32;
    const MAX: i32;
    const NOTCHES: &'static [(i32, Option<&'static str>)] = &[];
}

pub trait ToLuaExpr {
    fn to_lua_expr(&self) -> impl Display;
}

pub trait FromLuaOutput: Sized {
    fn from_lua_output(output: &str) -> Option<Self>;
}

pub struct LuaProperty<T> {
    pub module: &'static str,
    pub property: &'static str,
    _phantom: PhantomData<T>,
}

impl<T: Setting> LuaProperty<T> {
    pub const fn new(module: &'static str, property: &'static str) -> Self {
        LuaProperty { module, property, _phantom: PhantomData }
    }

    pub async fn get(&self, conn: &Connection) -> Result<T, LuaError> {
        let output = conn
            .eval_lua(&format!("require('{}').{}:get()", self.module, self.property))
            .await?;

        Ok(T::from_lua_output(&output).unwrap_or_default())
    }

    pub async fn set(&self, conn: &Connection, value: &T) -> Result<(), LuaError> {
        conn.eval_lua(&format!("require('{}').{}:set({})",
            self.module,
            self.property,
            value.to_lua_expr(),
        )).await?;

        Ok(())
    }
}
