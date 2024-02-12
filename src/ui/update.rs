use std::sync::Arc;

use adw::prelude::{ActionRowExt, PreferencesGroupExt, PreferencesPageExt};
use futures::StreamExt;
use glib::object::Cast;
use glib::types::StaticType;
use glib::WeakRef;
use gtk::{Align, FileDialog, FileFilter, Orientation};
use gtk::gio::{Cancellable, File};
use gtk::prelude::{BoxExt, ButtonExt, FileExt, WidgetExt};

use crate::firmware::Firmware;
use crate::flash::{self, FlashError, FlashStatus};
use crate::ui::application::DeviceContext;
use crate::ui::label_row::LabelRow;
use crate::ui::util::NavPageBuilder;
use crate::util::weak;

pub fn flow(device: DeviceContext) -> adw::NavigationPage {
    let nav = adw::NavigationView::new();

    nav.add(&select_firmware_page(UpdateContext {
        device,
        nav: weak(&nav),
    }));

    adw::NavigationPage::builder()
        .child(&nav)
        .build()
}

#[derive(Clone)]
struct UpdateContext {
    device: DeviceContext,
    nav: WeakRef<adw::NavigationView>,
}

fn select_firmware_page(ctx: UpdateContext) -> adw::NavigationPage {
    let page = adw::PreferencesPage::builder()
        .title("Update Firmware")
        .build();

    page.add(&select_group(ctx));

    NavPageBuilder::clamped(&page)
        .title(page.title().as_str())
        .build()
}

fn select_group(ctx: UpdateContext) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::new();

    let select_firmware = adw::ActionRow::builder()
        .title("Select firmware...")
        .activatable(true)
        .build();

    select_firmware.add_suffix(&gtk::Image::builder()
        .icon_name("companion-folder-symbolic")
        .build());

    select_firmware.connect_activated(move |widget| {
        let filter = FileFilter::new();
        filter.add_suffix("tra");

        let current_dir = std::env::current_dir().unwrap();

        let window: Option<gtk::Window> = widget
            .ancestor(gtk::Window::static_type())
            .and_then(|ancestor| ancestor.dynamic_cast().ok());

        FileDialog::builder()
            .default_filter(&filter)
            .initial_folder(&File::for_path(&current_dir))
            .title("Select Tangara firmware")
            .modal(true)
            .build()
            .open(window.as_ref(), Cancellable::NONE, {
                let ctx = ctx.clone();
                move |result| {
                    let Some(nav) = ctx.nav.upgrade() else { return };

                    let file = match result {
                        Ok(file) => file,
                        Err(error) => {
                            // TODO how do we surface this to user?
                            eprintln!("file dialog error: {error:?}");
                            return;
                        }
                    };

                    let Some(path) = file.path() else {
                        // no path?
                        eprintln!("no path in file from file dialog");
                        return;
                    };

                    match Firmware::open(&path) {
                        Ok(firmware) => {
                            let firmware = Arc::new(firmware);
                            nav.push(&review_firmware_page(ctx.clone(), firmware));
                        }
                        Err(error) => {
                            eprintln!("read firmware error: {}", error);
                        }
                    }
                }
            });
    });

    group.add(&select_firmware);

    group
}

fn review_firmware_page(ctx: UpdateContext, firmware: Arc<Firmware>) -> adw::NavigationPage {
    let intro_group = adw::PreferencesGroup::new();

    let intro_label = gtk::Label::builder()
        .label("Flashing new firmware to your Tangara will take a couple of minutes.\n\nTangara will reboot when flashing starts, and must remain plugged in until flashing is complete.")
        .wrap(true)
        .build();

    intro_group.add(&intro_label);

    let details_group = adw::PreferencesGroup::new();

    details_group.add(&*LabelRow::new("Firmware", &firmware.path().display().to_string()));
    details_group.add(&*LabelRow::new("Version", firmware.version()));

    let flash_group = adw::PreferencesGroup::new();

    let flash_button = gtk::Button::builder()
        .label("Flash!")
        .build();

    flash_button.connect_clicked({
        let ctx = ctx.clone();
        let firmware = firmware.clone();
        move |_| {
            let Some(nav) = ctx.nav.upgrade() else { return };

            nav.pop();
            nav.push(&flash_page(ctx.clone(), firmware.clone()));
        }
    });

    flash_group.add(&flash_button);

    let page = adw::PreferencesPage::builder()
        .title("Review Firmware")
        .build();

    page.add(&intro_group);
    page.add(&details_group);
    page.add(&flash_group);

    NavPageBuilder::clamped(&page)
        .title(page.title().as_str())
        .build()
}

fn flash_page(ctx: UpdateContext, firmware: Arc<Firmware>) -> adw::NavigationPage {
    let box_ = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .valign(Align::Center)
        .spacing(20)
        .build();

    box_.append(&gtk::Label::builder()
        .label("Do not disconnect Tangara")
        .build());

    let progress_bar = gtk::ProgressBar::builder()
        .build();

    let status_label = gtk::Label::builder()
        .build();

    box_.append(&progress_bar);
    box_.append(&status_label);

    let page = NavPageBuilder::clamped(&box_)
        .title("Flashing Tangara")
        .header(adw::HeaderBar::builder()
            .show_back_button(false)
            .show_end_title_buttons(false)
            .show_start_title_buttons(false)
            .build())
        .build();

    // lock the app global navigation while we're flashing
    let locked = ctx.device.nav.lock();

    // start flash now UI is built
    let mut flash = flash::start_flash(ctx.device.tangara.clone(), firmware);

    // progress handler
    glib::spawn_future_local(async move {
        let mut current_image = None;
        while let Some(progress) = flash.progress.next().await {
            match progress {
                FlashStatus::StartingFlash => {
                    status_label.set_label("Starting flash")
                }
                FlashStatus::Image(image) => {
                    status_label.set_label(&format!("Writing {image}..."));
                    progress_bar.set_fraction(0.0);
                    current_image = Some(image);
                }
                FlashStatus::Progress(written, total) => {
                    if total != 0 {
                        let progress = written as f64 / total as f64;
                        progress_bar.set_fraction(progress);
                    }

                    if let Some(image) = &current_image {
                        status_label.set_label(&format!("Writing {image}... block {written}/{total}"));
                    }
                }
            }
        }
    });

    // result channel
    glib::spawn_future_local(async move {
        let Some(nav) = ctx.nav.upgrade() else { return };

        let result = match flash.result.await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(Some(e)),
            Err(_) => Err(None),
        };

        nav.pop();
        nav.push(&complete(result));

        // hold on to locked until the flash has returned a result
        drop(locked);
    });

    page
}

fn complete(message: Result<(), Option<FlashError>>) -> adw::NavigationPage {
    let status_page = match message {
        Ok(()) => adw::StatusPage::builder()
            .icon_name("breeze-status-success")
            .title("Flash complete")
            .description("Please enjoy your freshly updated Tangara")
            .build(),
        Err(error) => adw::StatusPage::builder()
            .icon_name("companion-computer-sadface-symbolic")
            .title("Flash failed")
            .description(match error {
                Some(error) => format!("{error}"),
                None => "Unknown error".to_string(),
            })
            .build()
    };

    NavPageBuilder::clamped(&status_page)
        .title(status_page.title().as_str())
        .header(adw::HeaderBar::builder()
            .show_title(false)
            .build())
        .build()
}
