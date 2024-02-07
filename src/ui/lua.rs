use gtk::prelude::BoxExt;

use super::util::NavPageBuilder;

pub fn page() -> adw::NavigationPage {
    let entry = gtk::Entry::builder()
        .build();

    let box_ = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .valign(gtk::Align::Center)
        .spacing(20)
        .build();

    box_.append(&entry);

    NavPageBuilder::new(&box_)
        .title("Lua Console")
        .build()
}
