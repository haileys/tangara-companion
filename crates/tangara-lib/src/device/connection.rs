use core::slice;
use std::io::{self, Read, Write};
use std::string::FromUtf8Error;
use std::time::Duration;

use futures::channel::oneshot;
use mio_serial::{DataBits, FlowControl, SerialPort, SerialPortInfo, StopBits};
use thiserror::Error;

const CONSOLE_BAUD_RATE: u32 = 115200;
const CONSOLE_TIMEOUT: Duration = Duration::from_secs(1);
const MAX_CONSOLE_BUFFER: usize = 64*1024;
static CONSOLE_PROMPT: &[u8] = " → ".as_bytes();

#[derive(Debug, Error)]
pub enum OpenError {
    #[error("Opening serial port: {0}")]
    Port(#[from] mio_serial::Error),
    #[error(transparent)]
    Connection(#[from] ConnectionError),
    #[error("Connection thread terminated unexpectedly")]
    Canceled(#[from] oneshot::Canceled),
}

#[derive(Clone, Debug)]
pub struct Connection {
    tx: async_channel::Sender<Msg>,
}

#[allow(unused)]
#[derive(Debug, Error)]
#[error("lost connection")]
pub struct Disconnected;

#[derive(Debug, Error)]
pub enum LuaError {
    #[error(transparent)]
    Connection(#[from] Disconnected),
    #[error("invalid utf-8 in response")]
    InvalidUtf8(#[from] FromUtf8Error),
}

pub type SerialPortError = mio_serial::Error;

pub fn build_serial(serial_port: &SerialPortInfo) -> mio_serial::SerialPortBuilder {
    mio_serial::new(&serial_port.port_name, CONSOLE_BAUD_RATE)
        .data_bits(DataBits::Eight)
        .stop_bits(StopBits::One)
        .timeout(CONSOLE_TIMEOUT)
        .flow_control(FlowControl::None)
}

pub fn open_serial(serial_port: &SerialPortInfo) -> Result<Box<dyn SerialPort>, SerialPortError> {
    build_serial(serial_port).open()
}

impl Connection {
    pub async fn open(serial_port: &SerialPortInfo) -> Result<Connection, OpenError> {
        let port = open_serial(serial_port)?;
        let tx = start_connection(port).await?;

        Ok(Connection { tx })
    }

    async fn call<T, F>(&self, message: F) -> Result<T, Disconnected>
        where F: FnOnce(oneshot::Sender<T>) -> Msg
    {
        let (tx, rx) = oneshot::channel();

        self.tx.send(message(tx)).await.map_err(|_| Disconnected)?;

        rx.await.map_err(|_| Disconnected)
    }

    async fn console_command(&self, command: impl Into<String>) -> Result<Vec<u8>, Disconnected> {
        self.call(|tx| Msg::Command(command.into(), tx)).await
    }

    pub async fn firmware_version(&self) -> Result<String, LuaError> {
        let version = self
            .eval_lua("require('version').esp()")
            .await?;

        Ok(version)
    }

    pub async fn eval_lua(&self, code: &str) -> Result<String, LuaError> {
        if code.contains('\n') {
            panic!("newline in lua source code not allowed");
        }

        // escape the code for the console
        let code = code
            .replace("\\", "\\\\")
            .replace("\"", "\\\"");

        let command = format!("luarun \"io.stdout:write(({code}))\"");

        let result = self.console_command(command).await?;

        Ok(String::from_utf8(result)?)
    }

    pub async fn disconnect(&self) {
        // the only possible error we could encounter here is if the
        // connection is already disconnected. i'll prove it, see?
        let _: Result<(), Disconnected> =
            self.call(|tx| Msg::Disconnect(tx)).await;

        // so its safe to just ignore the result
    }
}

enum Msg {
    Disconnect(oneshot::Sender<()>),
    Command(String, oneshot::Sender<Vec<u8>>),
}

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("port error: {0}")]
    Port(#[from] mio_serial::Error),
    #[error("syncing to console: {0}")]
    Sync(#[from] SyncError),
}

async fn start_connection(port: Box<dyn SerialPort>)
    -> Result<async_channel::Sender<Msg>, OpenError>
{
    let (retn_tx, retn_rx) = oneshot::channel();

    std::thread::spawn(move || {
        let mut port = Protocol::new(Port::new(port));

        match port.sync() {
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
                eprintln!("error running tangara connection: {error:?}");
            }
        }
    });

    Ok(retn_rx.await??)
}

fn run_connection(
    cmd_rx: async_channel::Receiver<Msg>,
    mut port: Protocol,
) -> Result<(), ConnectionError> {
    while let Some(cmd) = cmd_rx.recv_blocking().ok() {
        match cmd {
            Msg::Disconnect(ret) => {
                // drop port first to ensure connection is closed
                drop(port);

                // then send response to command sender
                ret.send(()).ok();

                // and terminate the loop
                break;
            }
            Msg::Command(command, ret) => {
                port.sync()?;
                let result = port.execute_command(&command)?;
                ret.send(result).ok();
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
    Port(#[from] mio_serial::Error),
    #[error("received unexpected data, desync")]
    UnexpectedData { expected: u8, received: u8 },
    #[error("too much output")]
    TooMuchOutput,
}

struct Protocol {
    port: Port,
}

impl Protocol {
    pub fn new(port: Port) -> Self {
        Protocol { port }
    }

    pub fn sync(&mut self) -> Result<(), SyncError> {
        let in_buffer = self.port.port.bytes_to_read()?;
        if in_buffer > 0 {
            // read up anything that might still be in the buffer
            let mut _buffer = vec![0; in_buffer as usize];
            self.port.read_exact(&mut _buffer)?;
            self.port.flush_read();
        }

        self.port.write_all(b"\n")?;
        self.port.flush()?;
        self.read_until(CONSOLE_PROMPT)?;
        Ok(())
    }

    pub fn execute_command(&mut self, command: &str) -> Result<Vec<u8>, SyncError> {
        self.port.write_all(command.as_bytes())?;
        self.port.write_all(b"\n")?;
        self.port.flush()?;

        // read back the echoed command we just sent:
        for byte in command.as_bytes() {
            self.expect(*byte)?;
        }

        // tangara echoes LF as CRLF:
        self.expect(b'\r')?;
        self.expect(b'\n')?;

        // the rest of the output until the prompt is command output
        self.read_until(CONSOLE_PROMPT)
    }

    fn expect(&mut self, expected: u8) -> Result<(), SyncError> {
        let received = self.read_byte()?;
        if received == expected {
            Ok(())
        } else {
            Err(SyncError::UnexpectedData { expected, received })
        }
    }

    fn read_until(&mut self, delim: &[u8]) -> Result<Vec<u8>, SyncError> {
        let mut buff = Vec::new();

        loop {
            buff.push(self.read_byte()?);

            let suffix_idx = buff.len().saturating_sub(delim.len());

            if &buff[suffix_idx..] == delim {
                self.port.flush_read();
                buff.truncate(suffix_idx);
                return Ok(buff);
            }

            if buff.len() == MAX_CONSOLE_BUFFER {
                return Err(SyncError::TooMuchOutput);
            }
        }
    }

    /// Reads a single byte from the serial port. No point doing our own
    /// buffering here as the underlying implementation in the serialport
    /// only reads one byte at a time anyway
    fn read_byte(&mut self) -> io::Result<u8> {
        let mut byte = 0u8;
        self.port.read_exact(slice::from_mut(&mut byte))?;
        Ok(byte)
    }
}

struct Port {
    port: Box<dyn SerialPort>,
    // buffers for logging
    rx: Vec<u8>,
    tx: Vec<u8>,
}

impl Port {
    const MAX_BUFFER_LEN: usize = 1024;

    pub fn new(port: Box<dyn SerialPort>) -> Self {
        Port {
            port,
            rx: Vec::new(),
            tx: Vec::new(),
        }
    }

    pub fn flush_read(&mut self) {
        let rx = String::from_utf8_lossy(&self.rx);
        log::trace!("serial <-RX<-: {rx:?}");
        self.rx.clear();
    }
}

impl Read for Port {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.port.read(buf)?;
        self.rx.extend(buf[..n].iter().copied());
        self.rx.truncate(Self::MAX_BUFFER_LEN);
        Ok(n)
    }
}

impl Write for Port {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.port.write(buf)?;
        self.tx.extend(buf[..n].iter().copied());
        self.tx.truncate(Self::MAX_BUFFER_LEN);
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        let tx = String::from_utf8_lossy(&self.tx);
        log::trace!("serial ->TX->: {tx:?}");
        self.tx.clear();

        self.port.flush()
    }
}
