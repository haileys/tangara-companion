#![windows_subsystem = "windows"]

mod device;
mod firmware;
mod flash;
mod ui;
mod util;

use gtk::prelude::{ApplicationExt, ApplicationExtManual, GtkWindowExt};

use futures::StreamExt;

use device::Tangara;

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
            app.set_tangara(tangara).await;
        }
    });
}
