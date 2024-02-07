use gtk::{Align, ContentFit, Orientation};
use gtk::prelude::BoxExt;

pub fn page() -> adw::NavigationPage {
    let picture = gtk::Picture::for_resource("/zone/cooltech/tangara/companion/assets/logo.svg");
    picture.set_content_fit(ContentFit::ScaleDown);

    let label = gtk::Label::builder()
        .label("To begin, connect your Tangara and make sure it's switched on.")
        .build();

    let box_ = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .valign(Align::Center)
        .spacing(20)
        .build();

    box_.append(&picture);
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
