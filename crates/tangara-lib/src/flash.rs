use std::sync::Arc;

use espflash::{
    connection::{Connection, ResetAfterOperation, ResetBeforeOperation},
    flasher::Flasher,
    target::Chip,
    target::ProgressCallbacks,
};
use futures::channel::{mpsc, oneshot};
use serialport::FlowControl;
use thiserror::Error;

use crate::device::ConnectionParams;
use crate::firmware::{Firmware, Image};

const BAUD_RATE: u32 = 1000000;

pub enum FlashStatus {
    StartingFlash,
    Image(String),
    Progress(usize, usize),
}

pub struct Flash {
    pub progress: mpsc::Receiver<FlashStatus>,
    pub result: oneshot::Receiver<Result<(), FlashError>>,
}

pub fn setup(port: Arc<ConnectionParams>, firmware: Arc<Firmware>) -> (Flash, FlashTask) {
    let (progress_tx, progress) = mpsc::channel(32);
    let (result_tx, result) = oneshot::channel();

    let task = FlashTask {
        port,
        firmware,
        progress_tx,
        result_tx,
    };

    (Flash { progress, result }, task)
}

pub struct FlashTask {
    port: Arc<ConnectionParams>,
    firmware: Arc<Firmware>,
    progress_tx: mpsc::Sender<FlashStatus>,
    result_tx: oneshot::Sender<Result<(), FlashError>>,
}

impl FlashTask {
    /// Runs the flash - **blocks current thread**
    pub fn run(self) {
        let result = run_flash(&self.port, &self.firmware, self.progress_tx);
        let _ = self.result_tx.send(result);
    }
}

#[derive(Debug, Error)]
pub enum FlashError {
    #[error("opening usb serial interface: {0}")]
    OpenSerial(#[from] serialport::Error),
    #[error("connecting to device: {0}")]
    Connect(#[source] espflash::Error),
    #[error("writing image: {0}: {1}")]
    WriteBin(String, #[source] espflash::Error),
}

fn run_flash(
    port: &ConnectionParams,
    firmware: &Firmware,
    mut sender: mpsc::Sender<FlashStatus>,
) -> Result<(), FlashError> {
    let _ = sender.try_send(FlashStatus::StartingFlash);

    for image in firmware.images() {
        flash_image(&port, &image, &sender)?;
    }

    Ok(())
}

pub fn open_flash_connection(port: &ConnectionParams)
    -> Result<espflash::connection::Connection, serialport::Error>
{
    let connection_baud = 115_200;

    let serial = serialport::new(&port.serial.port_name, connection_baud)
        .flow_control(FlowControl::None)
        .open_native()?;

    Ok(Connection::new(
        serial,
        port.usb.clone(),
        ResetAfterOperation::HardReset,
        ResetBeforeOperation::DefaultReset,
        connection_baud,
    ))
}

fn flash_image(
    port: &ConnectionParams,
    image: &Image,
    sender: &mpsc::Sender<FlashStatus>,
) -> Result<(), FlashError> {
    let connection = open_flash_connection(port)?;

    let mut flasher = Flasher::connect(
        connection,
        true,
        false,
        false,
        Some(Chip::Esp32),
        Some(BAUD_RATE),
    )
    .map_err(FlashError::Connect)?;

    let mut progress = ProgressCallback {
        image: image.name.clone(),
        total: 0,
        sender: sender.clone(),
    };

    flasher
        .write_bin_to_flash(image.addr, &image.data, &mut progress)
        .map_err(|error| FlashError::WriteBin(image.name.clone(), error))?;

    Ok(())
}

struct ProgressCallback {
    image: String,
    total: usize,
    sender: mpsc::Sender<FlashStatus>,
}

impl ProgressCallbacks for ProgressCallback {
    fn init(&mut self, _: u32, total: usize) {
        self.total = total;
        let status = FlashStatus::Image(self.image.clone());
        let _ = self.sender.try_send(status);
    }

    fn update(&mut self, current: usize) {
        let status = FlashStatus::Progress(current, self.total);
        // use try_send here because it's ok if we drop a message
        let _ = self.sender.try_send(status);
    }

    fn finish(&mut self, _skipped: bool) {
        let status = FlashStatus::Progress(self.total, self.total);
        let _ = self.sender.try_send(status);
    }

    fn verifying(&mut self) {
        todo!()
    }
}
