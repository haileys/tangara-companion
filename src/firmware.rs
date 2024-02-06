use std::io::{Read, self};
use std::path::{Path, PathBuf};
use std::fs::File;

use thiserror::Error;
use zip::ZipArchive;
use zip::result::ZipError;

const MAX_IMAGE_SIZE: usize = 32 * 1024 * 1024;

pub struct Firmware {
    path: PathBuf,
    manifest: data::ManifestV0,
    images: Vec<Image>,
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
    #[error("Reading image: {0}: {1}")]
    ReadImage(String, #[source] ReadImageError),
}

#[derive(Debug, Error)]
pub enum ReadImageError {
    #[error("archive error: {0}")]
    NotFound(#[from] ZipError),
    #[error("image too large: {0} bytes")]
    TooLarge(u64),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

impl Firmware {
    pub fn open(path: &Path) -> Result<Self, OpenError> {
        let file = File::open(path).map_err(OpenError::Open)?;
        let mut zip = ZipArchive::new(file)?;
        let manifest = read_manifest(&mut zip)?;
        let images = read_images(&mut zip, &manifest.firmware)?;
        Ok(Firmware {
            path: path.to_owned(),
            manifest,
            images,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn version(&self) -> &str {
        &self.manifest.firmware.version
    }

    pub fn images(&self) -> &[Image] {
        &self.images
    }
}

pub struct Image {
    pub name: String,
    pub addr: u32,
    pub data: Vec<u8>,
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

fn read_manifest(zip: &mut ZipArchive<File>) -> Result<data::ManifestV0, OpenError> {
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

fn read_images(zip: &mut ZipArchive<File>, firmware: &data::Firmware)
    -> Result<Vec<Image>, OpenError>
{
    let mut images = Vec::new();

    for image in &firmware.images {
        let data = read_image_data(zip, &image.name).map_err(|error| {
            OpenError::ReadImage(image.name.clone(), error)
        })?;

        eprintln!("image {} @ {:x?}, {} bytes", image.name, image.addr, data.len());

        images.push(Image {
            name: image.name.clone(),
            addr: image.addr,
            data: data,
        });
    }

    Ok(images)
}

fn read_image_data(zip: &mut ZipArchive<File>, name: &str)
    -> Result<Vec<u8>, ReadImageError>
{
    let mut file = zip.by_name(name)?;

    let size = usize::try_from(file.size()).ok()
        .filter(|sz| *sz < MAX_IMAGE_SIZE)
        .ok_or_else(|| ReadImageError::TooLarge(file.size()))?;

    let mut data = vec![0; size];
    file.read_exact(&mut data)?;

    Ok(data)
}
