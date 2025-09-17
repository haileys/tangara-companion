use gtk::{Align, Orientation};
use gtk::prelude::BoxExt;
use tangara_lib::device::ConnectionParams;

use crate::ui;

pub fn page(params: &ConnectionParams) -> adw::NavigationPage {
    let logo = ui::widgets::logo::logo();

    let hbox = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .halign(Align::Center)
        .spacing(10)
        .build();

    let spinner = gtk::Spinner::new();
    spinner.start();

    let label = gtk::Label::builder()
        .label(format!("Connecting to Tangara at {}...", params.serial.port_name))
        .build();

    hbox.append(&spinner);
    hbox.append(&label);

    let box_ = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .valign(Align::Center)
        .spacing(20)
        .build();

    box_.append(&logo);
    box_.append(&hbox);

    let clamp = adw::Clamp::builder()
        .maximum_size(400)
        .child(&box_)
        .build();

    let header = adw::HeaderBar::builder()
        .show_title(false)
        .build();

    let view = adw::ToolbarView::builder()
        .content(&clamp)
        .build();

    view.add_top_bar(&header);

    adw::NavigationPage::builder()
        .title("Connecting to Tangara")
        .child(&view)
        .build()
}
