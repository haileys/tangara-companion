use std::io::Read;
use std::path::{Path, PathBuf};
use std::fs::File;

use thiserror::Error;
use zip::ZipArchive;
use zip::result::ZipError;

use self::data::ManifestV0;

pub struct Firmware {
    path: PathBuf,
    #[allow(unused)]
    zip: ZipArchive<File>,
    manifest: ManifestV0,
}

#[derive(Debug, Error)]
pub enum OpenError {
    #[error("Unable to open firmware archive: {0}")]
    Open(#[source] std::io::Error),
    #[error("Unreadable firmware archive: {0}")]
    Zip(#[from] ZipError),
    #[error("No manifest found in firmware archive")]
    NoManifest,
    #[error("Can't read firmware manifest: {0}")]
    ReadManifest(#[source] std::io::Error),
    #[error("Can't parse firmware manifest: {0}")]
    ParseManifest(#[source] serde_json::Error),
    #[error("Firmware archive newer than this version of Tangara Flasher supports")]
    UnsupportedVersion,
}

impl Firmware {
    pub fn open(path: &Path) -> Result<Self, OpenError> {
        let file = File::open(path).map_err(OpenError::Open)?;
        let mut zip = ZipArchive::new(file)?;
        let manifest = read_manifest(&mut zip)?;
        Ok(Firmware {
            path: path.to_owned(),
            zip,
            manifest,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn version(&self) -> &str {
        &self.manifest.firmware.version
    }
}

pub mod data {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Manifest {
        pub version: usize,
        pub data: serde_json::Value,
    }

    #[derive(Deserialize)]
    pub struct ManifestV0 {
        pub firmware: Firmware,
    }

    #[derive(Deserialize)]
    pub struct Firmware {
        pub version: String,
        pub images: Vec<FirmwareImage>,
    }

    #[derive(Deserialize)]
    pub struct FirmwareImage {
        pub addr: u32,
        pub name: String,
    }
}

fn read_manifest(zip: &mut ZipArchive<File>) -> Result<ManifestV0, OpenError> {
    let mut manifest_file = zip.by_name("tangaraflash.json").map_err(|error| {
        match error {
            ZipError::FileNotFound => OpenError::NoManifest,
            _ => OpenError::Zip(error),
        }
    })?;

    let mut manifest_json = String::new();
    manifest_file.read_to_string(&mut manifest_json)
        .map_err(OpenError::ReadManifest)?;

    let manifest = serde_json::from_str::<data::Manifest>(&manifest_json)
        .map_err(OpenError::ParseManifest)?;

    if manifest.version > 0 {
        return Err(OpenError::UnsupportedVersion);
    }

    let manifest = serde_json::from_value::<data::ManifestV0>(manifest.data)
        .map_err(OpenError::ParseManifest)?;

    Ok(manifest)
}
