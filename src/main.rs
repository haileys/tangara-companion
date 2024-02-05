#![windows_subsystem = "windows"]

mod device;
mod firmware;
mod flash;
mod ui;

use gtk::prelude::{ApplicationExt, ApplicationExtManual, GtkWindowExt};

use futures::StreamExt;

use device::Tangara;

#[allow(unused)]
use firmware::Firmware;

#[allow(unused)]
use flash::{FlashStatus, FlashError, Flash};

const APP_ID: &str = "zone.cooltech.tangara.Companion";

fn main() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(start);

    let args = std::env::args().collect::<Vec<_>>();
    app.run_with_args(&args)
}

fn start(app: &adw::Application) {
    let app = ui::Application::new(app);
    app.present();

    glib::spawn_future_local(async move {
        let mut watch = Tangara::watch();
        while let Some(tangara) = watch.next().await {
            app.set_tangara(tangara);
        }
    });
}

// fn about_devide(info: device::info::Info) -> adw::NavigationPage {
//     let header = adw::HeaderBar::builder()
//         .build();

//     let image = gtk::Picture::for_resource("/zone/cooltech/tangara/companion/logo.svg");

//     let box_ = gtk::Box::builder()
//         .orientation(Orientation::Vertical)
//         .valign(Align::Center)
//         .spacing(32)
//         .build();

//     box_.append(&image);
//     box_.append(&label);

//     let firmware = adw::PreferencesGroup::builder()
//         .build();

//     let clamp = adw::Clamp::builder()
//         .maximum_size(400)
//         .child(&box_)
//         .build();

//     let view = adw::ToolbarView::builder()
//         .content(&clamp)
//         .build();

//     view.add_top_bar(&header);

//     let content = adw::NavigationPage::builder()
//         .title("About Tangara")
//         .child(&view)
//         .build();

//     content
// }

// fn list_row(label: &str, value: &str) -> gtk::ListBoxRow
