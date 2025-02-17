use std::io::{self, ErrorKind, Write};
use std::process::ExitCode;

use console::{Key, Term};
use structopt::StructOpt;
use tangara_lib::device::connection::{open_serial, SerialPortError};
use thiserror::Error;
use serialport::SerialPort;

use crate::device;

#[derive(StructOpt)]
pub struct ConsoleOpt {}

#[derive(Error, Debug)]
pub enum ConsoleError {
    #[error(transparent)]
    FindTangara(#[from] device::FindError),
    #[error(transparent)]
    Open(#[from] SerialPortError),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

pub async fn run() -> Result<ExitCode, ConsoleError> {
    let mut term = Term::stdout();

    let device = device::find(&mut term).await?;
    let mut conn = open_serial(&device.params.serial)?;

    // spawn connection reader thread:
    std::thread::spawn({
        let conn = conn.try_clone()?;
        move || {
            if let Err(e) = run_rx(conn) {
                eprintln!("reader error! {e:?}");
            }
        }
    });

    // write initial newline for prompt
    conn.write_all(b"\n")?;

    loop {
        let key = match term.read_key() {
            Err(_) => break,
            Ok(Key::Unknown) => break,
            Ok(key) => key,
        };

        write_key(conn.as_mut(), key)?;
    }

    Ok(ExitCode::SUCCESS)
}

fn run_rx(mut conn: Box<dyn SerialPort>) -> io::Result<()> {
    let mut buff = [0u8; 1];

    loop {
        match conn.read(&mut buff) {
            Ok(0) => {
                // eof
                return Ok(());
            }
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::TimedOut => {
                // just a detail of the serialport crate
                // go around the loop
                continue;
            }
            Err(e) => { return Err(e) }
        }

        let mut stdout = std::io::stdout();
        stdout.write_all(&buff)?;
        stdout.flush()?;
    }
}

fn write_key(conn: &mut dyn SerialPort, key: Key) -> io::Result<()> {
    let mut buff = [0u8; 4];

    let bytes = match key {
        Key::Char(c) => c.encode_utf8(&mut buff).as_bytes(),
        Key::Enter => b"\n",
        Key::ArrowLeft => b"\x1b[D",
        Key::ArrowRight => b"\x1b[C",
        Key::ArrowUp => b"\x1b[A",
        Key::ArrowDown => b"\x1b[B",
        Key::End => b"\x1b[F",
        Key::Home => b"\x1b[H",
        Key::Tab => b"\t",
        Key::Del => b"\x1b[3~",
        Key::Backspace => &[8],
        k => {
            if cfg!(debug_assertions) {
                eprintln!("debug: unknown key: {k:?}");
            }
            return Ok(());
        }
    };

    conn.write_all(bytes)?;
    Ok(())
}
