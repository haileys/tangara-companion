use std::sync::Arc;

use futures::channel::{mpsc, oneshot};
use espflash::interface::Interface;
use espflash::flasher::{Flasher, ProgressCallbacks};
use thiserror::Error;

use crate::device::Tangara;
use crate::firmware::{Firmware, Image};

const BAUD_RATE: u32 = 1500000;

pub enum FlashStatus {
    StartingFlash,
    Image(String),
    Progress(usize, usize),
}

pub struct Flash {
    pub progress: mpsc::Receiver<FlashStatus>,
    pub result: oneshot::Receiver<Result<(), FlashError>>,
}

#[allow(unused)]
pub fn start_flash(port: Tangara, firmware: Arc<Firmware>) -> Flash {
    let (progress_tx, progress) = mpsc::channel(32);
    let (result_tx, result) = oneshot::channel();

    gtk::gio::spawn_blocking(move || {
        let result = run_flash(port, &firmware, progress_tx);
        let _ = result_tx.send(result);
    });

    Flash { progress, result }
}

#[derive(Debug, Error)]
pub enum FlashError {
    #[error("opening usb serial interface: {0}")]
    OpenInterface(String),
    #[error("connecting to device: {0}")]
    Connect(#[source] espflash::error::Error),
    #[error("writing image: {0}: {1}")]
    WriteBin(String, #[source] espflash::error::Error),
}

fn run_flash(
    port: Tangara,
    firmware: &Firmware,
    mut sender: mpsc::Sender<FlashStatus>,
) -> Result<(), FlashError> {
    let _ = sender.try_send(FlashStatus::StartingFlash);

    for image in firmware.images() {
        flash_image(&port, &image, &sender)?;
    }

    Ok(())
}

fn flash_image(
    tangara: &Tangara,
    image: &Image,
    sender: &mpsc::Sender<FlashStatus>
) -> Result<(), FlashError> {
    let interface = Interface::new(tangara.serial_port(), None, None)
        .map_err(|error| FlashError::OpenInterface(format!("{}", error)))?;

    let mut flasher = Flasher::connect(interface, tangara.usb_port().clone(), Some(BAUD_RATE), true)
        .map_err(FlashError::Connect)?;

    let mut progress = ProgressCallback {
        image: image.name.clone(),
        total: 0,
        sender: sender.clone(),
    };

    flasher.write_bin_to_flash(image.addr, &image.data, Some(&mut progress))
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

    fn finish(&mut self) {
        let status = FlashStatus::Progress(self.total, self.total);
        let _ = self.sender.try_send(status);
    }
}
