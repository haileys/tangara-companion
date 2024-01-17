use std::{path::Path, error::Error};

use async_channel::{Receiver, Sender};
use espflash::{interface::Interface, flasher::{Flasher, ProgressCallbacks}};
use serialport::{SerialPortType, SerialPortInfo, UsbPortInfo};
use thiserror::Error;

use crate::firmware::{Firmware, Image};

const USB_VID: u16 = 4617; // cool tech zone
const USB_PID: u16 = 8212; // Tangara
const BAUD_RATE: u32 = 1500000;

pub struct TangaraPort {
    serial: SerialPortInfo,
    usb: UsbPortInfo,
}

impl TangaraPort {
    pub fn port_name(&self) -> &str {
        &self.serial.port_name
    }
}

#[derive(Debug, Error)]
pub enum FindTangaraError {
    #[error("Error enumerating serial ports: {0}")]
    Port(#[from] serialport::Error),
    #[error("Can't find Tangara (is it plugged in?)")]
    NoTangara,
}

pub fn find_tangara() -> Result<TangaraPort, FindTangaraError> {
    for port in serialport::available_ports()? {
        if let SerialPortType::UsbPort(usb) = &port.port_type {
            if usb.vid == USB_VID && usb.pid == USB_PID {
                return Ok(TangaraPort {
                    serial: port.clone(),
                    usb: usb.clone(),
                });
            }
        }
    }

    Err(FindTangaraError::NoTangara)
}

pub enum FlashStatus {
    StartingFlash,
    Image(String),
    Progress(usize, usize),
    Error(FlashError),
    Complete,
}

pub fn start_flash(port: TangaraPort, firmware: Firmware) -> Receiver<FlashStatus> {
    let (tx, rx) = async_channel::bounded(32);

    gtk::gio::spawn_blocking(move || {
        match run_flash(port, firmware, tx.clone()) {
            Ok(()) => { let _ = tx.send_blocking(FlashStatus::Complete); }
            Err(error) => { let _ = tx.send_blocking(FlashStatus::Error(error)); }
        }
    });

    rx
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
    port: TangaraPort,
    firmware: Firmware,
    sender: Sender<FlashStatus>,
) -> Result<(), FlashError> {
    let _ = sender.send_blocking(FlashStatus::StartingFlash);

    for image in firmware.images() {
        flash_image(&port, &image, &sender)?;
    }

    Ok(())
}

fn flash_image(
    port: &TangaraPort,
    image: &Image,
    sender: &Sender<FlashStatus>
) -> Result<(), FlashError> {
    let interface = Interface::new(&port.serial, None, None)
        .map_err(|error| FlashError::OpenInterface(format!("{}", error)))?;

    let mut flasher = Flasher::connect(interface, port.usb.clone(), Some(BAUD_RATE), true)
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
    sender: Sender<FlashStatus>,
}

impl ProgressCallbacks for ProgressCallback {
    fn init(&mut self, addr: u32, total: usize) {
        self.total = total;
        let status = FlashStatus::Image(self.image.clone());
        let _ = self.sender.send_blocking(status);
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
