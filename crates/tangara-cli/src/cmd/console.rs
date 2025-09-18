use std::io;
use std::process::ExitCode;

use console::Term;
use structopt::StructOpt;
use tangara_lib::device::connection::{build_serial, SerialPortError};
use terminal::AsyncTerminal;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio_serial::SerialStream;

use crate::device;

mod terminal;

#[derive(StructOpt)]
pub struct ConsoleOpt {}

#[derive(Error, Debug)]
pub enum ConsoleError {
    #[error(transparent)]
    FindTangara(#[from] device::FindError),
    #[error(transparent)]
    Open(#[from] SerialPortError),
    #[error(transparent)]
    Io(#[from] io::Error),
}

pub async fn run() -> Result<ExitCode, ConsoleError> {
    let mut term = Term::stdout();

    let device = device::find(&mut term).await?;

    let mut serial = SerialStream::open(&build_serial(&device.params.serial))?;
    let mut terminal = AsyncTerminal::new(term);

    // send initial newline to get prompt to show:
    serial.write(b"\n").await?;

    tokio::io::copy_bidirectional(&mut serial, &mut terminal).await?;

    Ok(ExitCode::SUCCESS)
}
