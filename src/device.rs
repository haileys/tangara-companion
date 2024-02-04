pub mod connection;
pub mod info;

use std::io;
use std::time::Duration;

use futures::{Stream, SinkExt};
use futures::channel::{mpsc, oneshot};
use gtk::glib;
use serialport::{SerialPortInfo, UsbPortInfo, SerialPortType, SerialPortBuilder, SerialPort, DataBits, StopBits, FlowControl, ClearBuffer};
use thiserror::Error;

use self::connection::Connection;

const POLL_DURATION: Duration = Duration::from_secs(1);
const USB_VID: u16 = 4617; // cool tech zone
const USB_PID: u16 = 8212; // Tangara

#[derive(Clone)]
pub struct Tangara {
    serial: SerialPortInfo,
    usb: UsbPortInfo,
}

#[derive(Debug, Error)]
pub enum FindTangaraError {
    #[error("Error enumerating serial ports: {0}")]
    Port(#[from] serialport::Error),
    #[error("Can't find Tangara (is it plugged in?)")]
    NoTangara,
}

impl Tangara {
    pub fn serial_port_name(&self) -> &str {
        &self.serial.port_name
    }

    pub fn serial_port(&self) -> &SerialPortInfo {
        &self.serial
    }

    pub fn usb_port(&self) -> &UsbPortInfo {
        &self.usb
    }

    pub fn watch() -> impl Stream<Item = Result<Tangara, FindTangaraError>> {
        let (mut tx, rx) = mpsc::channel(1);

        glib::spawn_future_local(async move {
            loop {
                let result = Self::find().await;

                if let Err(_) = tx.send(result).await {
                    break;
                }

                // TODO - see if we can subscribe to hardware events or something?
                glib::timeout_future(POLL_DURATION).await;
            }
        });

        rx
    }

    pub async fn find() -> Result<Tangara, FindTangaraError> {
        for port in serialport::available_ports()? {
            if let SerialPortType::UsbPort(usb) = &port.port_type {
                if usb.vid == USB_VID && usb.pid == USB_PID {
                    return Ok(Tangara {
                        serial: port.clone(),
                        usb: usb.clone(),
                    });
                }
            }
        }

        Err(FindTangaraError::NoTangara)
    }

    pub fn open(&self) -> Result<Connection, connection::OpenError> {
        Connection::open(self.serial_port())
    }
}
