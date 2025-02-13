use std::process::ExitCode;

use console::{Term, style};
use structopt::StructOpt;
use thiserror::Error;

mod cmd;
mod device;

#[derive(StructOpt)]
pub struct Opt {
    #[structopt(subcommand)]
    cmd: Cmd,
}

#[derive(StructOpt)]
pub enum Cmd {
    Flash(cmd::flash::FlashOpt),
}

#[derive(Error, Debug)]
enum RunError {
    #[error(transparent)]
    Flash(#[from] cmd::flash::FlashError),
}

fn main() -> ExitCode {
    let opt = Opt::from_args();

    match thread_executor::block_on(run(opt)) {
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
        Cmd::Flash(args) => Ok(cmd::flash::run(args).await?),
    }
}
