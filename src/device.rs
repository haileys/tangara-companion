pub mod connection;
pub mod info;

use std::sync::Arc;
use std::time::Duration;

use futures::{Stream, SinkExt};
use futures::channel::mpsc;
use gtk::glib;
use serialport::{SerialPortInfo, UsbPortInfo, SerialPortType};
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

    pub fn watch() -> impl Stream<Item = Option<Arc<Tangara>>> {
        let (mut tx, rx) = mpsc::channel(1);

        glib::spawn_future_local(async move {
            let mut current = Self::find().await.ok().map(Arc::new);
            let _ = tx.send(current.clone()).await;

            while !tx.is_closed() {
                // TODO - see if we can subscribe to hardware events or something?
                glib::timeout_future(POLL_DURATION).await;

                let tangara = Self::find().await.ok();

                let current_name = current.as_deref().map(Tangara::serial_port_name);
                let tangara_name = tangara.as_ref().map(Tangara::serial_port_name);

                if current_name == tangara_name {
                    continue;
                }

                current = tangara.map(Arc::new);
                let _ = tx.send(current.clone()).await;
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

    pub async fn open(&self) -> Result<Connection, connection::OpenError> {
        Connection::open(self.serial_port()).await
    }
}
