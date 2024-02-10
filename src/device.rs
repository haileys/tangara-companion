pub mod connection;
pub mod info;

use std::sync::Arc;
use std::time::Duration;

use futures::{future, SinkExt, Stream, StreamExt};
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
    connection: Connection,
    params: Arc<ConnectionParams>,
}

#[derive(Clone)]
pub struct ConnectionParams {
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
    pub async fn open(params: &ConnectionParams)
        -> Result<Tangara, connection::OpenError>
    {
        let connection = Connection::open(&params.serial).await?;

        Ok(Tangara {
            connection,
            params: Arc::new(params.clone()),
        })
    }

    pub fn serial_port_name(&self) -> &str {
        &self.params.serial.port_name
    }

    pub fn serial_port(&self) -> &SerialPortInfo {
        &self.params.serial
    }

    pub fn usb_port(&self) -> &UsbPortInfo {
        &self.params.usb
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    fn watch_port() -> impl Stream<Item = Option<ConnectionParams>> {
        let (mut tx, rx) = mpsc::channel(1);

        glib::spawn_future_local(async move {
            let mut current = Self::find().await.ok();
            let _ = tx.send(current.clone()).await;

            while !tx.is_closed() {
                // TODO - see if we can subscribe to hardware events or something?
                glib::timeout_future(POLL_DURATION).await;

                let params = Self::find().await.ok();

                let current_port = current.as_ref().map(|p| &p.serial.port_name);
                let latest_port = params.as_ref().map(|p| &p.serial.port_name);

                if current_port == latest_port {
                    continue;
                }

                current = params;
                let _ = tx.send(current.clone());
            }
        });

        rx
    }

    pub fn watch() -> impl Stream<Item = Option<Tangara>> {

        Self::watch_port()
            .then(|params| async move {
                match params {
                    Some(params) => Tangara::open(&params).await.map(Some),
                    None => Ok(None),
                }
            })
            .filter_map(|result| future::ready(result
                .map_err(|error| { eprintln!("error opening tangara: {error:?}"); })
                .ok()))

        /*
        glib::spawn_future_local(async move {
            let mut current = Self::find().await.ok();
            let _ = tx.send(current.clone()).await;

            while !tx.is_closed() {
                // TODO - see if we can subscribe to hardware events or something?
                glib::timeout_future(POLL_DURATION).await;

                let params = Self::find().await.ok();

                let current_port = current.as_deref().map(|p| &p.serial.port_name);
                let latest_port = params.as_ref().map(|p| &p.serial.port_name);

                if current_port == latest_port {
                    continue;
                }

                let tangara = match params {
                    Some(params) => match Tangara::open(&params).await {
                        Ok(tangara) => Some(tangara),
                        Err(error) => {
                            eprintln!("error opening tangara: {error:?}");
                            continue;
                        }
                    }
                    None => None,
                };

                current = tangara;
                let _ = tx.send(current.clone()).await;
            }
        });

        rx
        */
    }

    pub async fn find() -> Result<ConnectionParams, FindTangaraError> {
        for port in serialport::available_ports()? {
            if let SerialPortType::UsbPort(usb) = &port.port_type {
                if usb.vid == USB_VID && usb.pid == USB_PID {
                    let params = ConnectionParams {
                        serial: port.clone(),
                        usb: usb.clone(),
                    };

                    return Ok(params);
                }
            }
        }

        Err(FindTangaraError::NoTangara)
    }
}
