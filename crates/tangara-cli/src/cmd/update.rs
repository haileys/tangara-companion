use std::io::{Write, self};
use std::process::ExitCode;

use console::{Term, style};
use indicatif::ProgressBar;
use semver::Version;
use serde::Deserialize;
use structopt::StructOpt;
use tempfile::NamedTempFile;
use thiserror::Error;

use crate::cmd::flash;
use crate::device;

#[derive(StructOpt)]
pub struct UpdateOpt {
    #[structopt(long)]
    force: bool,
}

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    FindTangara(#[from] device::FindError),
    #[error("downloading firmware archive: {0}")]
    SaveFirmware(#[source] io::Error),
    #[error("parsing latest release version: {0}")]
    ParseVersion(#[from] semver::Error),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("can't find url for latest firmware archive")]
    MissingAsset,
    #[error(transparent)]
    Flash(#[from] flash::FlashError),
}

pub async fn run(args: UpdateOpt) -> Result<ExitCode, UpdateError> {
    let mut term = Term::stdout();

    let device = device::find(&mut term).await?;
    let release = query_latest_release(&mut term).await?;

    match (&device.version, &release.version) {
        (Some(device), latest) if latest > device => {
            // latest version is newer than device version, proceed
        }
        (None, _) => {
            // cannot determine device version (eg. reflashing after broken flash), proceed
        }
        _ if args.force => {
            // forced update, proceed
        }
        _ => {
            writeln!(&mut term, "Tangara is up to date, there is nothing to do. Use {} to flash anyway",
                style("--force").bold())?;
            return Ok(ExitCode::SUCCESS);
        }
    }

    let firmware_file = download_firmware(&mut term, &release.url).await?;
    flash::flash(&mut term, firmware_file.path(), device).await?;
    Ok(ExitCode::SUCCESS)
}

async fn download_firmware(term: &mut Term, url: &str) -> Result<NamedTempFile, UpdateError> {
    let mut file = NamedTempFile::with_suffix(".tra").map_err(UpdateError::SaveFirmware)?;

    writeln!(term, "Downloading firmware from {}", style(url).blue())?;

    let mut response = reqwest::get(url).await?;

    let progress_bar = response.content_length()
        .map(ProgressBar::new)
        .unwrap_or_else(ProgressBar::no_length);

    while let Some(chunk) = response.chunk().await? {
        progress_bar.inc(chunk.len() as u64);
        file.write_all(&chunk).map_err(UpdateError::SaveFirmware)?;
    }

    progress_bar.finish_and_clear();

    Ok(file)
}

struct LatestRelease {
    version: Version,
    url: String,
}

async fn query_latest_release(term: &mut Term) -> Result<LatestRelease, UpdateError> {
    #[derive(Deserialize)]
    struct Release {
        name: String,
        assets: Vec<ReleaseAsset>,
    }

    #[derive(Deserialize)]
    struct ReleaseAsset {
        name: String,
        browser_download_url: String,
    }

    let url = "https://codeberg.org/api/v1/repos/cool-tech-zone/tangara-fw/releases/latest";

    writeln!(term, "Querying latest release from {}",
        style(url).blue())?;
    term.flush()?;

    let release: Release = reqwest::get(url).await?.json().await?;
    let version = release_version(&release.name)?;

    for asset in release.assets {
        if asset.name.ends_with(".tra") {
            return Ok(LatestRelease { version, url: asset.browser_download_url });
        }
    }

    Err(UpdateError::MissingAsset)
}

fn release_version(name: &str) -> Result<Version, semver::Error> {
    let version = if name.starts_with("v") {
        &name[1..]
    } else {
        &name
    };

    Version::parse(version)
}
