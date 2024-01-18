mod device;
mod firmware;
mod flash;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;

use device::Tangara;
use futures::StreamExt;
use gtk::gio::{Cancellable, File};
use gtk::pango::ffi::PANGO_SCALE;
use gtk::prelude::{ApplicationExt, ApplicationExtManual, GridExt, GtkWindowExt, ButtonExt, FileExt, WidgetExt};
use gtk::{Grid, Label, FileDialog, FileFilter, Align, ProgressBar};
use gtk::{glib, Button};

use firmware::Firmware;
use flash::{FlashStatus, FlashError};

const APP_ID: &str = "zone.cooltech.tangara.TangaraFlasher";

fn main() -> glib::ExitCode {
    if cfg!(target_os = "windows") {
        // we need larger stack on windows
        thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(app_main)
            .unwrap()
            .join()
            .unwrap()
    } else {
        app_main()
    }
}

fn app_main() -> glib::ExitCode {
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
            .valign(Align::Start)
            .build();

        window.set_resizable(false);
        window.present();

        App { window, nav }
    }
}

fn start(app: &adw::Application) {
    let app = App::new(app);
    app.nav.push(&welcome_page(app.clone()));
}

fn rows() -> Grid {
    Grid::builder()
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .margin_end(20)
        .row_spacing(20)
        .valign(Align::Start)
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
                                    app.nav.push(&firmware_page(app.clone(), FirmwarePage {
                                        firmware: Arc::new(firmware),
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

    let view = adw::ToolbarView::builder()
        .content(&layout)
        .valign(Align::Start)
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
    firmware: Arc<Firmware>,
}

fn firmware_page(app: App, page: FirmwarePage) -> adw::NavigationPage {
    let layout = rows();

    let path_label = Label::builder()
        .label(format!("Firmware: {}", page.firmware.path().display()))
        .halign(Align::Start)
        .build();

    let version_label = Label::builder()
        .label(format!("Version: {}", page.firmware.version()))
        .halign(Align::Start)
        .build();

    let status_label = Label::builder()
        .halign(Align::Start)
        .build();

    let flash_button = Button::builder()
        .label("Flash!")
        .sensitive(false)
        .build();

    layout.attach(&path_label, 0, 0, 1, 1);
    layout.attach(&version_label, 0, 1, 1, 1);
    layout.attach(&status_label, 0, 3, 1, 1);
    layout.attach(&flash_button, 0, 4, 1, 1);

    let tangara = Rc::new(RefCell::new(None));

    let task = glib::spawn_future_local({
        let tangara = tangara.clone();
        let flash_button = flash_button.clone();
        async move {
            let mut stream = Tangara::watch();

            while let Some(result) = stream.next().await {
                status_label.set_label(&match &result {
                    Ok(tangara) => format!("✅ Ready to flash Tangara at {}", tangara.port_name()),
                    Err(error) => format!("⚠️ {error}"),
                });

                flash_button.set_sensitive(result.is_ok());

                *tangara.borrow_mut() = result.ok().clone();
            }
        }
    });

    flash_button.connect_clicked({
        let tangara = tangara.clone();
        let firmware = page.firmware.clone();
        move |button| {
            let Some(tangara) = tangara.borrow().clone() else { return };

            button.set_sensitive(false);

            let status = flash::start_flash(tangara.clone(), firmware.clone());
            app.nav.push(&flash_page(app.clone(), FlashPage { status }));
        }
    });

    // make sure we cancel the background task when our UI goes away
    layout.connect_destroy(move |_| {
        eprintln!("destroying layout, destroying task!");
        task.abort();
    });

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
        .label("Do not disconnect Tangara")
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
                    app.nav.pop();
                    app.nav.push(&complete(Ok(())));
                    break;
                }
                Ok(FlashStatus::Error(error)) => {
                    app.nav.pop();
                    app.nav.push(&complete(Err(Some(error))));
                    break;
                }
                Err(_) => {
                    app.nav.pop();
                    app.nav.push(&complete(Err(None)));
                    break;
                }
            }
        }
    });

    let view = adw::ToolbarView::builder()
        .content(&layout)
        .build();

    let header = adw::HeaderBar::builder()
        .show_back_button(false)
        .show_end_title_buttons(false)
        .build();

    view.add_top_bar(&header);

    adw::NavigationPage::builder()
        .child(&view)
        .title("Flashing firmware")
        .build()
}

fn complete(message: Result<(), Option<FlashError>>) -> adw::NavigationPage {
    let layout = rows();

    let icon = Label::builder()
        .label(match &message {
            Ok(()) => "✅",
            Err(_) => "⚠️",
        })
        .build();

    if let Some(mut font) = icon.pango_context().font_description() {
        font.set_size(36 * PANGO_SCALE);
        icon.pango_context().set_font_description(Some(&font));
    }

    layout.attach(&icon, 0, 1, 1, 1);

    let text = Label::builder()
        .label(match &message {
            Ok(()) => "Please enjoy your freshly updated Tangara".to_owned(),
            Err(Some(error)) => format!("{error}"),
            Err(None) => "Unknown error".to_owned(),
        })
        .build();

    layout.attach(&text, 0, 2, 1, 1);

    let view = adw::ToolbarView::builder()
        .content(&layout)
        .build();

    let header = adw::HeaderBar::builder()
        .show_back_button(true)
        .build();

    view.add_top_bar(&header);

    adw::NavigationPage::builder()
        .child(&view)
        .title(match &message {
            Ok(()) => "Flash complete",
            Err(_) => "Flash failed",
        })
        .build()
}
