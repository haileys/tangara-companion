use core::slice;
use std::io;
use std::time::Duration;

use futures::channel::oneshot;
use serialport::{DataBits, FlowControl, SerialPort, SerialPortInfo, StopBits};
use thiserror::Error;

const CONSOLE_BAUD_RATE: u32 = 115200;
const CONSOLE_TIMEOUT: Duration = Duration::from_secs(1);
const EVENT_BUFFER: usize = 32;

#[derive(Debug)]
pub struct SerialConnection {
    port: Box<dyn SerialPort>,
    rx: async_channel::Receiver<Event>,
}

#[derive(Debug, Error)]
pub enum OpenError {
    #[error("Opening serial port: {0}")]
    Port(#[from] serialport::Error),
    #[error(transparent)]
    Connection(#[from] ConnectionError),
    #[error("Connection thread terminated unexpectedly")]
    Canceled(#[from] oneshot::Canceled),
}

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("port error: {0}")]
    Port(#[from] serialport::Error),
}

pub enum Event {
    UnframedLine(String),
    Frame(Frame),
}

pub struct Frame {
    pub opcode: Opcode,
    pub channel: ChannelId,
    pub data: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub enum Opcode {
    Data,
    Shutdown,
    Open,
}

impl Opcode {
    pub fn from_u8(byte: u8) -> Option<Opcode> {
        match byte & 0xf0 {
            0x00 => Some(Opcode::Data),
            0x10 => Some(Opcode::Shutdown),
            0x20 => Some(Opcode::Open),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ChannelId(pub u8);

impl ChannelId {
    pub fn from_u8(byte: u8) -> Self {
        ChannelId(byte & 0x0f)
    }
}

impl SerialConnection {
    pub async fn open(serial_port: &SerialPortInfo) -> Result<SerialConnection, OpenError> {
        let port = serialport::new(&serial_port.port_name, CONSOLE_BAUD_RATE)
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::One)
            .timeout(CONSOLE_TIMEOUT)
            .flow_control(FlowControl::None)
            .open()?;

        let rx = start_connection(port.try_clone()?).await?;

        Ok(SerialConnection { port, rx })
    }
}

fn start_connection(port: Box<dyn SerialPort>)
    -> Result<async_channel::Receiver<Event>, OpenError>
{
    let (tx, rx) = async_channel::bounded(EVENT_BUFFER);

    std::thread::spawn(move || {
        let port = ReadPort { port };
        match run_connection(tx, port) {
            Ok(()) => {}
            Err(error) => {
                // TODO signal this upwards with like an event or something
                eprintln!("error running tangara connection: {error:?}");
            }
        }
    });

    Ok(rx)
}

struct ReadPort {
    port: Box<dyn SerialPort>,
}

impl ReadPort {
    pub fn read_byte(&mut self) -> Result<u8, serialport::Error> {
        let mut byte = 0u8;
        // the underlying SerialPort implementation just reads byte-at-a-time,
        // even if a larger buffer is supplied, so just read byte-at-a-time
        // here too
        self.port.read_exact(slice::from_mut(&mut byte))?;
        Ok(byte)
    }
}

fn run_connection(
    tx: async_channel::Sender<Event>,
    mut port: ReadPort,
) -> Result<(), ConnectionError> {
    pub const FRAME_BEGIN: u8 = 0xfd;
    pub const FRAME_END: u8 = 0xfe;

    const MAX_UNFRAMED_LINE: usize = 1024;
    let mut unframed = Vec::new();

    loop {
        let byte = port.read_byte()?;
        if byte != FRAME_BEGIN {
            if unframed.len() < MAX_UNFRAMED_LINE {
                unframed.push(byte);
            }

            if byte == b'\n' {
                let line = String::from_utf8_lossy(&unframed).to_string();
                unframed.clear();

                if tx.send_blocking(Event::UnframedLine(line)).is_err() {
                    break;
                }
            }

            continue;
        }

        // beginning of framed data
        // read length
        let len = usize::from(port.read_byte()?);

        // read data, calculating expected checksum as we go
        let mut data = Vec::with_capacity(len);
        let mut cksum = 0u8;

        for _ in 0..len {
            let byte = port.read_byte()?;
            data.push(byte);
            cksum = cksum.wrapping_add(byte);
        }

        // read checksum byte
        let wire_cksum = port.read_byte()?;

        // read end frame byte
        if port.read_byte()? != FRAME_END {
            // invalid frame, discard
            continue;
        }

        // validate checksum
        if wire_cksum != cksum {
            continue;
        }

        if data.len() < 1 {
            continue;
        }

        let header = data.remove(0);
        let channel = ChannelId::from_u8(header);
        let Some(opcode) = Opcode::from_u8(header) else { continue };

        let frame = Frame {
            channel,
            opcode,
            data,
        };

        if tx.send_blocking(Event::Frame(frame)).is_err() {
            break;
        }
    }

    Ok(())
}
