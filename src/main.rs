pub mod firmware;
pub mod flash;

use std::cell::RefCell;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use faccess::{PathExt, AccessMode};
use firmware::Firmware;
use flash::{TangaraPort, FlashStatus};
use gtk::gio::{Cancellable, File};
use gtk::prelude::{ApplicationExt, ApplicationExtManual, GridExt, GtkWindowExt, ButtonExt, FileExt, WidgetExt};
use gtk::{Grid, Label, FileDialog, FileFilter, Align, ProgressBar};
use gtk::{glib, Button, Application, ApplicationWindow};
use thiserror::Error;

const APP_ID: &str = "zone.cooltech.tangara.TangaraFlasher";

fn main() -> glib::ExitCode {
    flash::find_tangara();

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
        .halign(Align::Start)
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

fn firmware_page(window: MainWindow, page: FirmwarePage) -> Grid {
    let layout = rows();

    let tangara = flash::find_tangara();

    let path_label = Label::builder()
        .label(format!("Firmware: {}", page.firmware.path().display()))
        .halign(Align::Start)
        .build();

    let version_label = Label::builder()
        .label(format!("Version: {}", page.firmware.version()))
        .halign(Align::Start)
        .build();

    layout.attach(&path_label, 0, 0, 1, 1);
    layout.attach(&version_label, 0, 1, 1, 1);

    if let Ok(port) = &tangara {
        let device_label = Label::builder()
            .label(format!("Device: {}", port.port_name()))
            .halign(Align::Start)
            .build();

        layout.attach(&device_label, 0, 2, 1, 1);
    }

    let status_label = Label::builder()
        .label(match &tangara {
            Ok(_) => "✅ Ready to flash".to_owned(),
            Err(error) => format!("⚠️ {error}"),
        })
        .halign(Align::Start)
        .build();

    let flash_button = Button::builder()
        .label("Flash!")
        .sensitive(tangara.is_ok())
        .build();

    flash_button.connect_clicked({
        let data = RefCell::new(tangara.ok().map(|tangara| {
            (tangara, page.firmware)
        }));

        move |button| {
            button.set_sensitive(false);

            if let Some((tangara, firmware)) = data.borrow_mut().take() {
                let status = flash::start_flash(tangara, firmware);
                let page = flash_page(window.clone(), FlashPage { status });
                window.set_child(Some(&page));
            }
        }
    });

    layout.attach(&status_label, 0, 3, 1, 1);
    layout.attach(&flash_button, 0, 4, 1, 1);

    layout
}

struct FlashPage {
    status: async_channel::Receiver<FlashStatus>,
}

fn flash_page(window: MainWindow, page: FlashPage) -> Grid {
    let layout = rows();

    let flashing_label = Label::builder()
        .label("Flashing... do not disconnect Tangara")
        .build();

    layout.attach(&flashing_label, 0, 0, 1, 1);

    let progress_bar = ProgressBar::builder()
        .build();

    layout.attach(&progress_bar, 0, 1, 1, 1);

    let status_label = Label::builder()
        .label("Status goes here")
        .build();

    layout.attach(&status_label, 0, 2, 1, 1);

    glib::spawn_future_local({
        let status = page.status;
        async move {
            loop {
                match status.recv().await {
                    Ok(FlashStatus::StartingFlash) => {
                        status_label.set_label("Starting flash")
                    }
                    Ok(FlashStatus::Image(image)) => {
                        status_label.set_label(&format!("Writing {image}..."));
                        progress_bar.set_fraction(0.0);
                    }
                    Ok(FlashStatus::Progress(written, total)) => {
                        if total != 0 {
                            let progress = written as f64 / total as f64;
                            progress_bar.set_fraction(progress);
                        }
                    }
                    Ok(FlashStatus::Complete) => {
                        let page = complete("✅ Flashing complete!");
                        window.set_child(Some(&page));
                        break;
                    }
                    Ok(FlashStatus::Error(error)) => {
                        let page = complete(&format!("⚠️ {error}"));
                        window.set_child(Some(&page));
                        break;
                    }
                    Err(_) => {
                        let page = complete("⚠️ Flasher terminated unexpectedly");
                        window.set_child(Some(&page));
                        break;
                    }
                }
            }
        }
    });

    layout
}

fn complete(message: &str) -> Grid {
    let layout = rows();

    let label = Label::builder()
        .label(message)
        .build();

    layout.attach(&label, 0, 1, 1, 1);

    layout
}

#[derive(Debug, Error)]
#[error("Error looking for Tangara in /dev/serial/by-id: {0}")]
struct FindTangaraError(#[from] io::Error);

fn find_tangara() -> Result<Option<PathBuf>, FindTangaraError> {
    for entry in std::fs::read_dir("/dev/serial/by-id/")? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if !name.starts_with("usb-cool_tech_zone_Tangara_") { continue }
        if !name.ends_with("-if00") { continue }
        return Ok(Some(entry.path()));
    }
    Ok(None)
}

#[derive(Debug, Error)]
enum CheckTangaraError {
    #[error("{0}")]
    Find(#[from] FindTangaraError),
    #[error("Can't find Tangara (is it plugged in?)")]
    NotFound,
    #[error("Tangara is plugged in, but we don't have permission to flash it")]
    NoPermissions,
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
}

fn check_tangara() -> Result<PathBuf, CheckTangaraError> {
    let path = find_tangara()?.ok_or(CheckTangaraError::NotFound)?;

    match path.access(AccessMode::READ | AccessMode::WRITE) {
        Ok(()) => Ok(path),
        Err(e) if e.kind() == ErrorKind::NotFound => Err(CheckTangaraError::NotFound),
        Err(e) if e.kind() == ErrorKind::PermissionDenied => Err(CheckTangaraError::NoPermissions),
        Err(e) => Err(e.into()),
    }
}
