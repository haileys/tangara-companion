use std::cell::Cell;
use std::rc::{Rc, Weak};
use std::sync::Arc;

use adw::prelude::BinExt;
use derive_more::Deref;
use glib::WeakRef;
use gtk::prelude::{GridExt, WidgetExt, ListBoxRowExt};

use crate::device::Tangara;
use crate::ui;

use super::application::DeviceContext;

#[derive(Deref)]
pub struct MainView {
    #[deref]
    split: adw::NavigationSplitView,
    sidebar: Sidebar,
    controller: NavController,
}

impl MainView {
    pub fn new() -> Self {
        let split = adw::NavigationSplitView::new();

        let controller = NavController::new(&split);

        let sidebar = Sidebar::new();
        split.set_sidebar(Some(&*sidebar));
        split.set_content(Some(&ui::welcome::page()));

        MainView {
            split,
            sidebar,
            controller,
        }
    }

    pub async fn set_device(&self, device: Option<Arc<Tangara>>) {
        match device {
            None => {
                self.sidebar.device_nav.set_child(None::<&gtk::Widget>);
                self.split.set_content(Some(&ui::welcome::page()));
            }
            Some(tangara) => {
                let list = DeviceNavBuilder::new(tangara, self.controller.clone())
                    .add_item(
                        "Overview",
                        "help-about-symbolic",
                        move |device| ui::overview::page(device),
                    )
                    .add_item(
                        "Lua Console",
                        "",
                        move |_| ui::lua::page(),
                    )
                    .add_item(
                        "Firmware Update",
                        "software-update-available-symbolic",
                        |device| ui::update::flow(device),
                    )
                    .build();

                self.sidebar.device_nav.set_child(Some(&list));
            }
        }
    }
}

#[derive(Deref)]
pub struct Sidebar {
    #[deref]
    sidebar: adw::NavigationPage,
    device_nav: adw::Bin,
}

impl Sidebar {
    pub fn new() -> Self {
        let device_nav = adw::Bin::new();

        let view = adw::ToolbarView::builder()
            .content(&device_nav)
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
            device_nav,
        }
    }
}

#[derive(Clone)]
pub struct NavController {
    split: WeakRef<adw::NavigationSplitView>,
}

impl NavController {
    pub fn new(split: &adw::NavigationSplitView) -> Self {
        let ref_ = WeakRef::new();
        ref_.set(Some(split));
        NavController { split: ref_ }
    }

    pub fn present(&self, page: &adw::NavigationPage) {
        if let Some(split) = self.split.upgrade() {
            split.set_content(Some(page));
        }
    }
}

struct DeviceNavBuilder {
    list: gtk::ListBox,
    actions: Vec<Box<dyn Fn(DeviceContext) -> adw::NavigationPage>>,
    context: DeviceContext,
    controller: NavController,
}

impl DeviceNavBuilder {
    pub fn new(tangara: Arc<Tangara>, controller: NavController) -> Self {
        let list = gtk::ListBox::builder()
            .css_classes(["navigation-sidebar"])
            .build();

        let context = DeviceContext {
            tangara,
            nav: DeviceNavController::new(&list),
        };

        DeviceNavBuilder {
            list,
            actions: Vec::new(),
            context,
            controller,
        }
    }

    pub fn add_item<Func, Page>(mut self, label: &str, icon: &str, action: Func) -> Self
        where
            Func: Fn(DeviceContext) -> Page + 'static,
            Page: Into<adw::NavigationPage>,
    {
        self.list.append(&sidebar_row(label, icon));
        self.actions.push(Box::new(move |ctx| action(ctx).into()));
        self
    }

    pub fn build(self) -> gtk::ListBox {
        let list = self.list;

        list.connect_row_activated({
            let actions = self.actions;
            let context = self.context;
            let controller = self.controller;
            move |_, row| {
                let Ok(index) = usize::try_from(row.index()) else { return };
                let Some(action) = actions.get(index) else { return };
                let page = action(context.clone());
                controller.present(&page);
            }
        });

        // activate default:
        if let Some(row) = list.row_at_index(0) {
            row.activate();
        }

        list
    }
}

pub struct DeviceNavController {
    list: WeakRef<gtk::ListBox>,
    lock_count: Cell<usize>,
}

impl DeviceNavController {
    pub fn new(list: &gtk::ListBox) -> Rc<Self> {
        let ref_ = WeakRef::new();
        ref_.set(Some(list));

        Rc::new(DeviceNavController {
            list: ref_,
            lock_count: Cell::new(0),
        })
    }

    pub fn lock(self: Rc<Self>) -> DeviceNavLocked {
        let Some(list) = self.list.upgrade() else {
            return DeviceNavLocked { nav: Weak::new() };
        };

        let count = self.lock_count.get();
        self.lock_count.set(count + 1);
        list.set_sensitive(false);

        let nav = Rc::downgrade(&self);
        DeviceNavLocked { nav }
    }
}

pub struct DeviceNavLocked {
    nav: Weak<DeviceNavController>,
}

impl Drop for DeviceNavLocked {
    fn drop(&mut self) {
        let Some(nav) = self.nav.upgrade() else { return };
        let Some(list) = nav.list.upgrade() else { return };

        let count = nav.lock_count.get();
        let count = count.saturating_sub(1);
        nav.lock_count.set(count);

        if count == 0 {
            list.set_sensitive(true);
        }
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
