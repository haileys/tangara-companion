use gtk::prelude::{BoxExt, EditableExt, EntryExt, WidgetExt};

use crate::ui::util::NavPageBuilder;

mod entry;

mod highlight;

pub fn page() -> adw::NavigationPage {
    let console = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .valign(gtk::Align::End)
        .build();

    let header = adw::HeaderBar::new();
    let footer = footer();

    let view = adw::ToolbarView::builder()
        .content(&console)
        .build();

    view.add_top_bar(&header);
    view.add_bottom_bar(&footer);

    adw::NavigationPage::builder()
        .child(&view)
        .title("Lua Console")
        .build()
}

fn footer() -> gtk::Box {
    let entry = entry::entry();

    let footer = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .css_classes(["toolbar"])
        .build();

    footer.append(&entry);

    footer
}
