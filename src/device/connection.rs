use std::io;
use std::time::Duration;

use futures::channel::oneshot;
use serialport::{SerialPortInfo, SerialPort, DataBits, StopBits, FlowControl, ClearBuffer};
use thiserror::Error;

// const CONSOLE_BAUD_RATE: u32 = 115200;
const CONSOLE_BAUD_RATE: u32 = 115200;
const CONSOLE_TIMEOUT: Duration = Duration::from_secs(1);
const MAX_CONSOLE_BUFFER: usize = 64*1024;
static CONSOLE_PROMPT: &[u8] = " → ".as_bytes();

#[derive(Debug, Error)]
pub enum OpenError {
    #[error("Opening serial port: {0}")]
    Port(#[from] serialport::Error),
    #[error(transparent)]
    Connection(#[from] ConnectionError),
    #[error("Connection thread terminated unexpectedly")]
    Canceled(#[from] oneshot::Canceled),
}

#[derive(Clone)]
pub struct Connection {
    tx: async_channel::Sender<Cmd>,
}

#[allow(unused)]
#[derive(Debug, Error)]
pub enum CommandError {
    #[error("lost connection")]
    Disconnected,
    #[error("invalid response")]
    InvalidResponse,
}

#[derive(Debug, Error)]
pub enum ExecuteLuaError {
    #[error("lost connection")]
    Disconnected,
    #[error("response not valid utf-8")]
    InvalidUtf8,
    #[error("invalid response")]
    InvalidResponse,
}

impl Connection {
    pub async fn open(serial_port: &SerialPortInfo) -> Result<Connection, OpenError> {
        let port = serialport::new(&serial_port.port_name, CONSOLE_BAUD_RATE)
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::One)
            .timeout(CONSOLE_TIMEOUT)
            .flow_control(FlowControl::Hardware)
            .open()?;

        let tx = start_connection(port).await?;

        Ok(Connection { tx })
    }

    // pub async fn execute_command(&self, command: &[&str]) -> Result<Vec<u8>, CommandError> {
    //     let command
    // }

    pub async fn eval_lua(&self, code: &str) -> Result<String, ExecuteLuaError> {
        if code.contains('\n') {
            panic!("newline in lua soure code not allowed");
        }

        let (tx, rx) = oneshot::channel();

        self.tx.send(Cmd::ExecLua(code.to_owned(), tx)).await
            .map_err(|_| ExecuteLuaError::Disconnected)?;

        let result = rx.await
            .map_err(|_| ExecuteLuaError::Disconnected)?;

        // split response into lines
        let output = std::str::from_utf8(&result)
            .map_err(|_| ExecuteLuaError::InvalidUtf8)?;

        let (_, response) = output.split_once("\r\n")
            .ok_or(ExecuteLuaError::InvalidResponse)?;

        let mut response = response.to_owned();

        if response.ends_with("\r\n") {
            response.truncate(response.len() - 2);
        }

        Ok(response)
    }
}

// fn escape_command_arg(out: &mut String, arg: &str) {
//     let need_quote = arg.contains(char::is_whitespace);

//     if need_quote {
//         out += '"';
//     }

//     for c in arg {
//         match c {
//             '"' => { out += "\\\"" }

//         }
//     }
// }

enum Cmd {
    ExecLua(String, oneshot::Sender<Vec<u8>>),
}

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("port error: {0}")]
    Port(#[from] serialport::Error),
    #[error("syncing to console: {0}")]
    Sync(#[from] SyncError),
}

async fn start_connection(mut port: Box<dyn SerialPort>)
    -> Result<async_channel::Sender<Cmd>, OpenError>
{
    let (retn_tx, retn_rx) = oneshot::channel();

    std::thread::spawn(move || {
        match sync(&mut port) {
            Ok(()) => {}
            Err(error) => {
                let _ = retn_tx.send(Err(ConnectionError::Sync(error)));
                return;
            }
        }

        let (tx, rx) = async_channel::bounded(32);
        let _ = retn_tx.send(Ok(tx));

        match run_connection(rx, port) {
            Ok(()) => {}
            Err(error) => {
                // TODO signal this upwards with like an event or something
                eprintln!("error running tangara connection: {error}");
            }
        }
    });

    Ok(retn_rx.await??)
}

fn run_connection(
    cmd_rx: async_channel::Receiver<Cmd>,
    mut port: Box<dyn SerialPort>,
) -> Result<(), ConnectionError> {
    while let Some(cmd) = cmd_rx.recv_blocking().ok() {
        match cmd {
            Cmd::ExecLua(code, ret) => {
                // escape the code for the console
                let code = code
                    .replace("\\", "\\\\")
                    .replace("\"", "\\\"");

                let code = format!("io.stdout:write(({code})..'\\n')");

                // resync to console
                sync(&mut port)?;

                println!("LUA: {code:?}");

                // write our command:
                port.write_all(b"luarun \"")?;
                port.write_all(code.as_bytes())?;
                port.write_all(b"\"\n")?;
                port.flush()?;

                // read the response
                let output = read_and_sync(&mut port)?;
                let _ = ret.send(output);
            }
        }
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("port error: {0}")]
    Port(#[from] serialport::Error),
    #[error("unexpected eof")]
    Eof,
    #[error("too much output")]
    TooMuchOutput,
}

fn sync(port: &mut Box<dyn SerialPort>) -> Result<(), SyncError> {
    port.clear(ClearBuffer::Input)?;
    port.write_all(b"\n")?;
    port.flush()?;

    read_and_sync(port)?;
    Ok(())
}

fn read_and_sync(port: &mut Box<dyn SerialPort>) -> Result<Vec<u8>, SyncError> {
    let mut output = Vec::new();
    let mut buff = [0u8; 64];

    loop {
        // println!("output -> {output:?}");
        // println!("       -> {:?}", String::from_utf8_lossy(&output));
        let n = port.read(&mut buff)?;

        // if the read ever EOFs we've lost the connection
        if n == 0 {
            return Err(SyncError::Eof);
        }

        // otherwise write output into our buffer
        output.extend(&buff[..n]);

        // we have a limit on how much data we will read from the console
        // before giving up on reaching sync. if we exceed that, just error
        if output.len() > MAX_CONSOLE_BUFFER {
            return Err(SyncError::TooMuchOutput);
        }

        // if the output ends with the console prompt then we have read the
        // full output and reached sync
        if output.ends_with(CONSOLE_PROMPT) {
            // remove prompt from end
            output.truncate(output.len() - CONSOLE_PROMPT.len());
            // and return
            return Ok(output);
        }
    }
}
