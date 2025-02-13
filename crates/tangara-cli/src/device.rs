use std::io::Write;

use console::{Term, style};
use thiserror::Error;

use tangara_lib::device::{ConnectionParams, Tangara};

#[derive(Error, Debug)]
pub enum FindError {
    #[error(transparent)]
    FindTangara(#[from] tangara_lib::device::FindTangaraError),
}

pub async fn find(term: &mut Term) -> Result<ConnectionParams, FindError> {
    let params = Tangara::find()?;

    match tangara_version(&params).await {
        Ok(version) => {
            let _ = writeln!(term, "Found Tangara at {}, current firmware version {}",
                style(&params.serial.port_name).green(),
                style(&version).bold());
        }
        Err(error) => {
            let _ = writeln!(term, "Found Tangara at {}, cannot retrieve current firmware information: {}",
                style(&params.serial.port_name).green(),
                style(&format!("{error}")).yellow());
        }
    }

    Ok(params)
}

#[derive(Debug, Error)]
enum VersionError {
    #[error(transparent)]
    OpenTangara(#[from] tangara_lib::device::connection::OpenError),
    #[error(transparent)]
    FindVersion(#[from] tangara_lib::device::connection::LuaError),
}

async fn tangara_version(params: &ConnectionParams) -> Result<String, VersionError> {
    let tangara = Tangara::open(&params).await?;
    let version = tangara.connection().firmware_version().await?;
    Ok(version)
}
