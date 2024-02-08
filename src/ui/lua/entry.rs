use std::cell::RefCell;

use gtk::prelude::{EditableExt, WidgetExt, EntryExt};

use crate::ui::lua::highlight::Highlight;

pub fn entry(highlight: Highlight) -> gtk::Entry {
    let entry = gtk::Entry::builder()
        .css_classes(["console-input"])
        .hexpand(true)
        .build();

    entry.connect_changed({
        move |entry| {
            let attrs = highlight.process(entry.text().as_str());
            entry.set_attributes(&attrs);
        }
    });

    entry
}
