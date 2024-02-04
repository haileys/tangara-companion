use super::connection::{Connection, ExecuteLuaError};

#[derive(Debug)]
pub struct Info {
    pub version: Version,
    pub collation: String,
    pub database: Database,
}

#[derive(Debug)]
pub struct Version {
    pub firmware: String,
    pub samd: String,
}

#[derive(Debug)]
pub struct Database {
    pub schema_version: String,
    pub disk_size: Option<usize>,
}

pub type InfoError = ExecuteLuaError;

pub async fn get(conn: &Connection) -> Result<Info, InfoError> {
    Ok(Info {
        version: Version {
            firmware: conn.eval_lua("require('version').esp()").await?,
            samd: conn.eval_lua("require('version').samd()").await?,
        },
        collation: conn.eval_lua("require('version').collator()").await?,
        database: Database {
            schema_version: conn.eval_lua("require('database').version()").await?,
            disk_size: conn.eval_lua("require('database').size()").await?.parse().ok(),
        },
    })
}
