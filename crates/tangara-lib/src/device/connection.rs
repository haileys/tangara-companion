use std::io;
use std::time::Duration;

use futures::channel::oneshot;
use serialport::{DataBits, FlowControl, SerialPort, SerialPortInfo, StopBits};
use thiserror::Error;

const CONSOLE_BAUD_RATE: u32 = 115200;
const CONSOLE_TIMEOUT: Duration = Duration::from_secs(1);

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

pub enum Frame {
    Data(ChannelId, Vec<u8>),
    Shutdown(ChannelId),
    Open(ChannelId, String),
}

pub struct ChannelId(pub u8);

impl SerialConnection {
    pub async fn open(serial_port: &SerialPortInfo) -> Result<SerialConnection, OpenError> {
        let port = serialport::new(&serial_port.port_name, CONSOLE_BAUD_RATE)
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::One)
            .timeout(CONSOLE_TIMEOUT)
            .flow_control(FlowControl::None)
            .open()?;

        let tx = start_connection(port.try_clone()?).await?;

        Ok(SerialConnection { tx })
    }
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
