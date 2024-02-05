#![windows_subsystem = "windows"]

mod device;
mod firmware;
mod flash;
mod ui;

use gtk::prelude::{ApplicationExt, ApplicationExtManual, GridExt, GtkWindowExt};

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
    let sidebar = sidebar();
    let content = ui::TngWelcomePage::new();

    let split = adw::NavigationSplitView::builder()
        .sidebar(&sidebar)
        .content(&content)
        .build();

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .content(&split)
        .width_request(280)
        .height_request(200)
        .default_width(800)
        .default_height(600)
        .build();

    window.present();

    glib::spawn_future_local(async move {
        let tangara = Tangara::find().await.unwrap();
        let connection = tangara.open().unwrap();
        let info = device::info::get(&connection).await.unwrap();
        println!("result -> {info:#?}");
    });
}

fn sidebar() -> adw::NavigationPage {
    let list = gtk::ListBox::builder()
        .css_classes(["navigation-sidebar"])
        .build();

    list.append(&sidebar_row("About", "/zone/cooltech/tangara/companion/icon-info.svg"));
    list.append(&sidebar_row("Firmware", "/zone/cooltech/tangara/companion/icon-firmware.svg"));

    let header = adw::HeaderBar::builder()
        .build();

    let view = adw::ToolbarView::builder()
        .content(&list)
        .build();

    view.add_top_bar(&header);

    let sidebar = adw::NavigationPage::builder()
        .title("")
        .child(&view)
        .build();

    sidebar
}

fn sidebar_row(
    label_text: &str,
    icon_resource: &str,
) -> gtk::ListBoxRow {
    let grid = gtk::Grid::builder()
        .build();

    let icon = gtk::Image::from_resource(icon_resource);
    grid.attach(&icon, 1, 1, 1, 1);

    let label = gtk::Label::builder()
        .label(label_text)
        .css_classes(["label"])
        .build();

    grid.attach(&label, 2, 1, 1, 1);

    gtk::ListBoxRow::builder()
        .child(&grid)
        .build()
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
