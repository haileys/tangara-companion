use std::{sync::{Arc, Mutex};
use std::time::Duration;

use serialport::SerialPort;
use async_channel::{Receiver, Sender};
use thiserror::Error;

use super::protocol::ChannelId;

const BAUD_RATE: u32 = 115200;
const TIMEOUT: Duration = Duration::from_secs(1);
const EVENT_BUFFER: usize = 32;

pub struct Connection {
    shared: Arc<Shared>,
}

#[derive(Debug, Error)]
pub enum OpenError {
    #[error(transparent)]
    Serial(tokio_serial::Error),
}

pub struct ChannelSender {
    channel: ChannelId,
    shared: Arc<Shared>,
}

pub struct ChannelReceiver {
    channel: ChannelId,
    shared: Arc<Shared>,
}

struct Shared {
    state: Mutex<State>,
}

struct State {
    dropped_senders: Vec<ChannelId>,
    channels: HashMap<ChannelId
}

struct Channel {
    tx: LineState,
    rx: Sender<Vec<u8>>,
}

enum LineState {
    Open,
    Closing,
    Closed,
}

impl Connection {
    pub async fn new(path: &Path) -> Result<Connection, OpenError> {
        let port = tokio_serial::new(path, BAUD_RATE)
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::One)
            .timeout(CONSOLE_TIMEOUT)
            .flow_control(FlowControl::None)
            .open()?;

        tokio::task::spawn_local(run(port))
    }
}

async fn run(port: Box<dyn SerialPort>) {

}
