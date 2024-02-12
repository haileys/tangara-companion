#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod device;
mod firmware;
mod flash;
mod ui;
mod util;

use gtk::prelude::{ApplicationExt, ApplicationExtManual, GtkWindowExt};

use futures::StreamExt;

use device::Tangara;
use log::LevelFilter;

const APP_ID: &str = "zone.cooltech.tangara.companion";

fn main() -> glib::ExitCode {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .parse_default_env()
        .init();

    tangara_companion_resources::init();

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
        let mut watch = Box::pin(Tangara::watch());
        while let Some(tangara) = watch.next().await {
            app.set_tangara(tangara).await;
        }
    });
}
