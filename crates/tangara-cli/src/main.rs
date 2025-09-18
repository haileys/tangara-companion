use std::process::ExitCode;

use console::{Term, style};
use log::LevelFilter;
use structopt::StructOpt;
use thiserror::Error;

mod cmd;
mod device;
mod util;

#[derive(StructOpt)]
pub struct Opt {
    #[structopt(subcommand)]
    cmd: Cmd,
}

#[derive(StructOpt)]
pub enum Cmd {
    Console(cmd::console::ConsoleOpt),
    Flash(cmd::flash::FlashOpt),
    Update(cmd::update::UpdateOpt),
}

#[derive(Error, Debug)]
enum RunError {
    #[error(transparent)]
    Console(#[from] cmd::console::ConsoleError),
    #[error(transparent)]
    Flash(#[from] cmd::flash::FlashError),
    #[error(transparent)]
    Update(#[from] cmd::update::UpdateError),
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let opt = Opt::from_args();

    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .filter(Some("mio_serial"), LevelFilter::Info)
        .parse_default_env()
        .init();

    match run(opt).await {
        Ok(code) => code,
        Err(error) => {
            let term = Term::stdout();
            let _ = term.write_line(&format!("{} {}",
                style("error:").red().bold(),
                style(&format!("{error}")).bold()));

            ExitCode::FAILURE
        }
    }
}

async fn run(opt: Opt) -> Result<ExitCode, RunError> {
    match opt.cmd {
        Cmd::Console(_) => Ok(cmd::console::run().await?),
        Cmd::Flash(args) => Ok(cmd::flash::run(args).await?),
        Cmd::Update(args) => Ok(cmd::update::run(args).await?),
    }
}
