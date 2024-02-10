use super::connection::{Connection, LuaError};

#[derive(Debug)]
pub struct Info {
    pub firmware: Firmware,
    pub database: Database,
}

#[derive(Debug)]
pub struct Firmware {
    pub version: String,
    pub samd: String,
    pub collation: String,
}

#[derive(Debug)]
pub struct Database {
    pub schema_version: String,
    pub disk_size: Option<u64>,
}

pub type InfoError = LuaError;

pub async fn get(conn: &Connection) -> Result<Info, InfoError> {
    Ok(Info {
        firmware: Firmware {
            version: conn.eval_lua("require('version').esp()").await?,
            samd: conn.eval_lua("require('version').samd()").await?,
            collation: conn.eval_lua("require('version').collator()").await?,
        },
        database: Database {
            schema_version: conn.eval_lua("require('database').version()").await?,
            disk_size: conn.eval_lua("require('database').size()").await?.parse().ok(),
        },
    })
}
