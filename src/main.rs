pub mod firmware;

use std::io::{self, ErrorKind};
use std::path::Path;
use std::rc::Rc;

use faccess::{PathExt, AccessMode};
use firmware::Firmware;
use gtk::gio::{Cancellable, File};
use gtk::prelude::{ApplicationExt, ApplicationExtManual, GridExt, GtkWindowExt, ButtonExt, FileExt};
use gtk::{Grid, Label, FileDialog, FileFilter};
use gtk::{glib, Button, Application, ApplicationWindow};

const APP_ID: &str = "zone.cooltech.tangara.TangaraFlasher";
const TANGARA_DEV: &str = "/dev/serial/by-id/usb-TinyUSB_TinyUSB_Device_123456-if00";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_window);
    app.run()
}

type MainWindow = Rc<ApplicationWindow>;

fn build_window(app: &Application) {
    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Tangara Flasher")
        .build();

    let window = Rc::new(window);

    let welcome = welcome_page(window.clone());
    window.set_child(Some(&welcome));

    window.present();
}

fn rows() -> Grid {
    Grid::builder()
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .margin_end(20)
        .row_spacing(20)
        .build()
}

fn welcome_page(window: MainWindow) -> Grid {
    let layout = rows();

    let welcome_label = Label::builder()
        .label("Welcome to Tangara Flasher! Select a firmware archive to get started :)\n\nHint: It will probably be called something like tangarafw.tra")
        .build();

    let select_firmware_button = Button::builder()
        .label("Select firmware...")
        .build();

    select_firmware_button.connect_clicked(move |_| {
        let filter = FileFilter::new();
        filter.add_pattern("*.tra");

        let current_dir = std::env::current_dir().unwrap();

        FileDialog::builder()
            .default_filter(&filter)
            .initial_folder(&File::for_path(&current_dir))
            .title("Select Tangara firmware")
            .modal(true)
            .build()
            .open(Some(&*window), Cancellable::NONE, {
                let window = window.clone();
                move |result| {
                    match result {
                        Ok(file) => {
                            let Some(path) = file.path() else {
                                // no path?
                                eprintln!("no path in file from file dialog");
                                return;
                            };

                            match Firmware::open(&path) {
                                Ok(firmware) => {
                                    let page = firmware_page(window.clone(), FirmwarePage {
                                        firmware,
                                    });

                                    window.set_child(Some(&page));
                                }
                                Err(error) => {
                                    eprintln!("read firmware error: {}", error);
                                }
                            }
                        }
                        Err(error) => {
                            // TODO how do we surface this to user?
                            eprintln!("file dialoag error: {error:?}");
                        }
                    }
                }
            });
    });

    layout.attach(&welcome_label, 0, 1, 1, 1);
    layout.attach(&select_firmware_button, 0, 2, 1, 1);

    layout
}

struct FirmwarePage {
    firmware: Firmware,
}

fn firmware_page(_window: MainWindow, page: FirmwarePage) -> Grid {
    let layout = rows();

    let path_label = Label::builder()
        .label(format!("Firmware: {}", page.firmware.path().display()))
        .build();

    let version_label = Label::builder()
        .label(format!("Version: {}", page.firmware.version()))
        .build();

    let device_status = check_device_status();

    let status_label = Label::builder()
        .label(match &device_status {
            DeviceStatus::Ready => "✅ Ready to flash".to_owned(),
            DeviceStatus::NotFound => "⚠️ Can't find Tangara (is it plugged in?)".to_owned(),
            DeviceStatus::NoPermissions => "⚠️ Tangara is plugged in, but we don't have permission to flash it".to_owned(),
            DeviceStatus::IoError(e) => format!("⚠️ Error: {e}"),
        })
        .build();

    let flash_button = Button::builder()
        .label("Flash!")
        .sensitive(match &device_status {
            DeviceStatus::Ready => true,
            _ => false,
        })
        .build();

    layout.attach(&path_label, 0, 0, 1, 1);
    layout.attach(&version_label, 0, 1, 1, 1);
    layout.attach(&status_label, 0, 2, 1, 1);
    layout.attach(&flash_button, 0, 3, 1, 1);

    layout
}

enum DeviceStatus {
    Ready,
    NotFound,
    NoPermissions,
    IoError(io::Error),
}

fn check_device_status() -> DeviceStatus {
    let dev_path = Path::new(TANGARA_DEV);
    match dev_path.access(AccessMode::READ | AccessMode::WRITE) {
        Ok(()) => DeviceStatus::Ready,
        Err(e) if e.kind() == ErrorKind::NotFound => DeviceStatus::NotFound,
        Err(e) if e.kind() == ErrorKind::PermissionDenied => DeviceStatus::NoPermissions,
        Err(e) => DeviceStatus::IoError(e),
    }
}
