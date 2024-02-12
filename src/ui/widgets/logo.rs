use std::cell::RefCell;

use glib::{clone, object::ObjectExt};
use gtk::prelude::WidgetExt;

pub fn logo() -> gtk::Picture {
    let style_manager = adw::StyleManager::default();

    let picture = gtk::Picture::for_resource(resource_path(&style_manager));
    picture.set_content_fit(gtk::ContentFit::ScaleDown);

    // connect to notify::dark signal, being careful not to hold a strong
    // reference to the picture which would hold it alive for as long as
    // style_manager is alive (the whole program lifetime)
    let signal_id = style_manager.connect_dark_notify(
        clone!(@weak picture => move |style_manager| {
            picture.set_resource(Some(resource_path(&style_manager)))
        })
    );

    // when picture is destroyed, disconnect the signal so that we don't
    // leak useless signal connections either
    picture.connect_destroy({
        let signal_id = RefCell::new(Some(signal_id));
        clone!(@weak style_manager => move |_| {
            if let Some(signal_id) = signal_id.borrow_mut().take() {
                style_manager.disconnect(signal_id);
            }
        })
    });

    picture
}

fn resource_path(style: &adw::StyleManager) -> &'static str {
    if style.is_dark() {
        "/zone/cooltech/tangara/companion/assets/logo-dark.svg"
    } else {
        "/zone/cooltech/tangara/companion/assets/logo.svg"
    }
}
