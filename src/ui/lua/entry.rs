use std::cell::RefCell;

use gtk::prelude::{EditableExt, WidgetExt, EntryExt};

use crate::ui::lua::highlight::Highlight;

pub fn entry() -> gtk::Entry {
    let entry = gtk::Entry::builder()
        .build();

    let mut font = entry.pango_context()
        .font_description()
        .unwrap_or_default();

    font.set_family("monospace");

    entry.connect_changed({
        let highlight = RefCell::new(Highlight::new(font));

        move |entry| {
            let attrs = highlight.borrow_mut().process(entry.text().as_str());
            entry.set_attributes(&attrs);
        }
    });

    entry
}
