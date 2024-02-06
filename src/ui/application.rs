use std::rc::Rc;

use adw::prelude::BinExt;
use derive_more::Deref;
use gtk::prelude::GridExt;

use crate::ui;
use crate::device::{self, Tangara};

#[derive(Deref)]
pub struct Application {
    #[deref]
    window: adw::ApplicationWindow,
    split: adw::NavigationSplitView,
    sidebar: Sidebar,
    welcome: ui::WelcomePage,
}

impl Application {
    pub fn new(app: &adw::Application) -> Self {
        let split = adw::NavigationSplitView::new();

        let sidebar = Sidebar::new();
        split.set_sidebar(Some(&*sidebar));

        let welcome = ui::WelcomePage::new();
        split.set_content(Some(&*welcome));

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .content(&split)
            .width_request(400)
            .height_request(400)
            .default_width(800)
            .default_height(800)
            .build();

        Application {
            window,
            split,
            sidebar,
            welcome,
        }
    }

    pub async fn set_tangara(&self, tangara: Option<Rc<Tangara>>) {
        let Some(tangara) = tangara else {
            self.sidebar.clear();
            self.split.set_content(Some(&*self.welcome));
            return;
        };

        // TODO show loading spinner screen in here maybe?

        let conn = tangara.open().unwrap();
        let info = device::info::get(&conn).await.unwrap();
        let content = ui::DevicePage::new(&tangara, &info);

        self.sidebar.show();
        self.split.set_content(Some(&*content));
    }
}

#[derive(Deref)]
struct Sidebar {
    #[deref]
    sidebar: adw::NavigationPage,
    nav: adw::Bin,
    device_nav: gtk::ListBox,
}

impl Sidebar {
    pub fn new() -> Self {
        let device_nav = gtk::ListBox::builder()
            .css_classes(["navigation-sidebar"])
            .build();

        device_nav.append(&sidebar_row("Information", "help-about-symbolic"));
        device_nav.append(&sidebar_row("Update", "software-update-available-symbolic"));

        let nav = adw::Bin::new();

        let view = adw::ToolbarView::builder()
            .content(&nav)
            .build();

        view.add_top_bar(
            &adw::HeaderBar::builder()
                .build());

        let sidebar = adw::NavigationPage::builder()
            .title("Tangara Companion")
            .child(&view)
            .build();

        Sidebar {
            sidebar,
            nav,
            device_nav,
        }
    }

    pub fn clear(&self) {
        self.nav.set_child(None::<&gtk::Widget>);
    }

    pub fn show(&self) {
        self.nav.set_child(Some(&self.device_nav));
    }
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
