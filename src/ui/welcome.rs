use gtk::{Align, Orientation};
use gtk::prelude::BoxExt;

use crate::ui;

pub fn page() -> adw::NavigationPage {
    let logo = ui::widgets::logo::logo();

    let label = gtk::Label::builder()
        .label("To begin, connect your Tangara and make sure it's switched on.")
        .build();

    let box_ = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .valign(Align::Center)
        .spacing(20)
        .build();

    box_.append(&logo);
    box_.append(&label);

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
        .title("Welcome to Tangara")
        .child(&view)
        .build()
}
