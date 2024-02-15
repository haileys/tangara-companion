pub mod connection;
pub mod info;

use std::sync::Arc;

use serialport::{SerialPortInfo, UsbPortInfo, SerialPortType};
use thiserror::Error;

use crate::{flash::{Flash, FlashTask, self}, firmware};

use self::{connection::{Connection, LuaError}, info::Firmware};

const USB_VID: u16 = 4617; // cool tech zone
const USB_PID: u16 = 8212; // Tangara

#[derive(Clone, Debug)]
pub struct Tangara {
    connection: Connection,
    params: Arc<ConnectionParams>,
}

#[derive(Clone, Debug)]
pub struct ConnectionParams {
    pub serial: SerialPortInfo,
    pub usb: UsbPortInfo,
}

#[derive(Debug, Error)]
pub enum FindTangaraError {
    #[error("Error enumerating serial ports: {0}")]
    Port(#[from] serialport::Error),
    #[error("Can't find Tangara, make sure it's plugged in and turned on")]
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

    pub fn setup_flash(self, firmware: Arc<firmware::Firmware>) -> (Flash, FlashTask) {
        let params = self.params.clone();

        // drop our own connection before trying to reopen the port for flash
        drop(self);

        flash::setup(params, firmware)
    }
}
