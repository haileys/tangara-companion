use std::rc::Rc;

use derive_more::Deref;
use gtk::prelude::GridExt;

use crate::ui;
use crate::device::Tangara;
use crate::device::info;

#[derive(Deref)]
pub struct Application {
    #[deref]
    window: adw::ApplicationWindow,
    split: adw::NavigationSplitView,
}

impl Application {
    pub fn new(app: &adw::Application) -> Self {
        let sidebar = sidebar();
        let content = ui::WelcomePage::new();

        let split = adw::NavigationSplitView::builder()
            .sidebar(&sidebar)
            .content(&*content)
            .build();

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .content(&split)
            .width_request(400)
            .height_request(400)
            .default_width(800)
            .default_height(800)
            .build();

        Application { window, split }
    }

    pub fn set_tangara(&self, tangara: Option<Rc<Tangara>>) {
        match tangara {
            None => {
                let content = ui::WelcomePage::new();
                self.split.set_content(Some(&*content));
            }
            Some(tangara) => {
                let split = self.split.clone();

                glib::spawn_future_local(async move {
                    let conn = tangara.open().unwrap();
                    let info = info::get(&conn).await.unwrap();
                    let content = ui::DevicePage::new(&tangara, &info);
                    split.set_content(Some(&*content));
                });
            }
        }
    }
}

fn sidebar() -> adw::NavigationPage {
    let list = gtk::ListBox::builder()
        .css_classes(["navigation-sidebar"])
        .build();

    list.append(&sidebar_row("About", "help-about-symbolic"));
    list.append(&sidebar_row("Firmware", "software-update-available-symbolic"));

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
    icon_name: &str,
) -> gtk::ListBoxRow {
    let grid = gtk::Grid::builder()
        .valign(gtk::Align::Center)
        .column_spacing(12)
        .margin_bottom(12)
        .margin_top(12)
        .margin_start(6)
        .margin_end(6)
        .build();

    let icon = gtk::Image::from_icon_name(icon_name);
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
