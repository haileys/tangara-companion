pub mod firmware;
pub mod flash;

use std::cell::RefCell;

use gtk::gio::{Cancellable, File};
use gtk::prelude::{ApplicationExt, ApplicationExtManual, GridExt, GtkWindowExt, ButtonExt, FileExt, WidgetExt};
use gtk::{Grid, Label, FileDialog, FileFilter, Align, ProgressBar};
use gtk::{glib, Button};

use firmware::Firmware;
use flash::FlashStatus;

const APP_ID: &str = "zone.cooltech.tangara.TangaraFlasher";

fn main() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(start);
    app.run()
}

#[derive(Clone)]
struct App {
    window: adw::ApplicationWindow,
    nav: adw::NavigationView,
}

impl App {
    pub fn new(app: &adw::Application) -> Self {
        let nav = adw::NavigationView::builder()
            .pop_on_escape(true)
            .build();

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .content(&nav)
            .build();

        window.present();

        App { window, nav }
    }

    pub fn push(&self, page: adw::NavigationPage) {
        self.nav.push(&page);
    }
}

fn start(app: &adw::Application) {
    let app = App::new(app);
    app.push(welcome_page(app.clone()));
}

fn rows() -> Grid {
    Grid::builder()
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .margin_end(20)
        .row_spacing(20)
        .hexpand(true)
        .column_homogeneous(true)
        .build()
}

fn welcome_page(app: App) -> adw::NavigationPage {
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
            .open(Some(&app.window), Cancellable::NONE, {
                let app = app.clone();
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
                                    app.push(firmware_page(app.clone(), FirmwarePage {
                                        firmware,
                                    }));
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

    let clamp = adw::Clamp::builder()
        .child(&layout)
        .build();

    let view = adw::ToolbarView::builder()
        .content(&clamp)
        .build();

    let header = adw::HeaderBar::builder()
        .build();

    view.add_top_bar(&header);

    adw::NavigationPage::builder()
        .child(&view)
        .title("Tangara Flasher")
        .build()
}

struct FirmwarePage {
    firmware: Firmware,
}

fn firmware_page(app: App, page: FirmwarePage) -> adw::NavigationPage {
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
                app.push(flash_page(app.clone(), FlashPage { status }));
            }
        }
    });

    layout.attach(&status_label, 0, 3, 1, 1);
    layout.attach(&flash_button, 0, 4, 1, 1);

    let view = adw::ToolbarView::builder()
        .content(&layout)
        .build();

    let header = adw::HeaderBar::builder()
        .show_back_button(true)
        .build();

    view.add_top_bar(&header);

    adw::NavigationPage::builder()
        .child(&view)
        .title("Review firmware")
        .build()
}

struct FlashPage {
    status: async_channel::Receiver<FlashStatus>,
}

fn flash_page(app: App, page: FlashPage) -> adw::NavigationPage {
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

    glib::spawn_future_local(async move {
        let mut current_image = None;
        loop {
            match page.status.recv().await {
                Ok(FlashStatus::StartingFlash) => {
                    status_label.set_label("Starting flash")
                }
                Ok(FlashStatus::Image(image)) => {
                    status_label.set_label(&format!("Writing {image}..."));
                    progress_bar.set_fraction(0.0);
                    current_image = Some(image);
                }
                Ok(FlashStatus::Progress(written, total)) => {
                    if total != 0 {
                        let progress = written as f64 / total as f64;
                        progress_bar.set_fraction(progress);
                    }

                    if let Some(image) = &current_image {
                        status_label.set_label(&format!("Writing {image}... block {written}/{total}"));
                    }
                }
                Ok(FlashStatus::Complete) => {
                    app.push(complete("✅ Flashing complete!"));
                    break;
                }
                Ok(FlashStatus::Error(error)) => {
                    app.push(complete(&format!("⚠️ {error}")));
                    break;
                }
                Err(_) => {
                    app.push(complete("⚠️ Flasher terminated unexpectedly"));
                    break;
                }
            }
        }
    });

    adw::NavigationPage::builder()
        .child(&layout)
        .title("Flashing firmware")
        .build()
}

fn complete(message: &str) -> adw::NavigationPage {
    let layout = rows();

    let label = Label::builder()
        .label(message)
        .build();

    layout.attach(&label, 0, 1, 1, 1);

    adw::NavigationPage::builder()
        .child(&layout)
        .title("Tangara Flasher")
        .build()
}
