use std::io::Write;

use console::{Term, style};
use semver::Version;
use thiserror::Error;

use tangara_lib::device::{ConnectionParams, Tangara};

#[derive(Error, Debug)]
pub enum FindError {
    #[error(transparent)]
    FindTangara(#[from] tangara_lib::device::FindTangaraError),
}

pub struct FoundDevice {
    pub params: ConnectionParams,
    pub version: Option<Version>,
}

pub async fn find(term: &mut Term) -> Result<FoundDevice, FindError> {
    let params = Tangara::find()?;

    let version = match tangara_version(&params).await {
        Ok(version) => {
            let _ = writeln!(term, "Found Tangara at {}, current firmware version {}",
                style(&params.serial.port_name).green(),
                style(&version).bold());
            Some(version)
        }
        Err(error) => {
            let _ = writeln!(term, "Found Tangara at {}, cannot retrieve current firmware information: {}",
                style(&params.serial.port_name).green(),
                style(&format!("{error}")).yellow());
            None
        }
    };

    Ok(FoundDevice { params, version })
}

#[derive(Debug, Error)]
enum VersionError {
    #[error(transparent)]
    OpenTangara(#[from] tangara_lib::device::connection::OpenError),
    #[error(transparent)]
    FindVersion(#[from] tangara_lib::device::connection::LuaError),
    #[error("parsing device firmware version: {0}")]
    ParseVersion(#[from] semver::Error),
}

async fn tangara_version(params: &ConnectionParams) -> Result<Version, VersionError> {
    let tangara = Tangara::open(&params).await?;
    let version = tangara.connection().firmware_version().await?;
    Ok(Version::parse(&version)?)
}
