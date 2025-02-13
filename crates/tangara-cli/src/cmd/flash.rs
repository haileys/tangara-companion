use std::io::{Write, self};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use console::{Term, style};
use futures::StreamExt;
use indicatif::ProgressBar;
use structopt::StructOpt;
use tangara_lib::flash::{FlashStatus, self};
use thiserror::Error;

use tangara_lib::device::{Tangara, ConnectionParams};
use tangara_lib::firmware::Firmware;

use crate::device;

#[derive(StructOpt)]
pub struct FlashOpt {
    image: PathBuf,
}

pub async fn run(args: FlashOpt) -> Result<ExitCode, FlashError> {
    match flash(args).await {
        // turn writeln! io errors into failure exits:
        Err(FlashError::Io(_)) => Ok(ExitCode::FAILURE),
        // pass thru all other results:
        Ok(code) => Ok(code),
        Err(other) => Err(other),
    }
}

#[derive(Error, Debug)]
pub enum FlashError {
    #[error("opening firmware: {0}")]
    Firmware(#[from] tangara_lib::firmware::OpenError),
    #[error(transparent)]
    FindTangara(#[from] device::FindError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Flash(#[from] tangara_lib::flash::FlashError),
}


async fn flash(args: FlashOpt) -> Result<ExitCode, FlashError> {
    let mut term = Term::stdout();

    let firmware = Firmware::open(&args.image).map(Arc::new)?;
    let params = device::find(&mut term).await?;

    // show confirmation prompt
    write!(&term, "Flash version {} to device? [y/n] ",
        style(firmware.version()).bold())?;
    term.flush()?;

    // read key and echo it
    let char = term.read_char()?;
    write!(&term, "{char}")?;
    term.flush()?;

    // check user response
    match char {
        'y' | 'Y' => {}
        _ => { return Ok(ExitCode::FAILURE); }
    }

    writeln!(&term)?;

    let progress_bar = ProgressBar::new(1);
    progress_bar.set_message("Starting flash");

    let (mut flash, task) = flash::setup(Arc::new(params), firmware);
    std::thread::spawn(move || task.run());

    while let Some(progress) = flash.progress.next().await {
        match progress {
            FlashStatus::StartingFlash => {}
            FlashStatus::Image(image) => {
                progress_bar.set_message(image);
                progress_bar.set_position(0);
                progress_bar.set_length(1);
            }
            FlashStatus::Progress(written, total) => {
                progress_bar.set_length(total as u64);
                progress_bar.set_position(written as u64);
            }
        }
    }

    progress_bar.finish_and_clear();

    flash.result.await.unwrap()?;

    writeln!(&term, "{}", style("Flash success!").green())?;

    Ok(ExitCode::SUCCESS)
}
