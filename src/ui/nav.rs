use std::cell::Cell;
use std::error::Error;
use std::rc::{Rc, Weak};

use adw::prelude::BinExt;
use derive_more::Deref;
use glib::WeakRef;
use gtk::prelude::{BoxExt, ButtonExt, GridExt, ListBoxRowExt, WidgetExt};

use tangara_lib::device::{ConnectionParams, Tangara};

use crate::ui;
use crate::ui::application::DeviceContext;
use crate::ui::util::NavPageBuilder;

use super::application::DeviceErrorChoice;
use super::update;
use super::util::SendOnce;

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

        let view = MainView {
            split,
            sidebar,
            controller,
        };

        view
    }

    fn show_page_without_sidebar(&self, page: &adw::NavigationPage) {
        self.sidebar.device_nav.set_child(None::<&gtk::Widget>);
        self.split.set_content(Some(page));
    }

    pub fn show_welcome(&self) {
        self.show_page_without_sidebar(&ui::welcome::page());
    }

    pub fn show_connecting(&self, params: &ConnectionParams) {
        self.show_page_without_sidebar(&ui::spinner::connect(params));
    }

    pub fn show_rebooting(&self, params: &ConnectionParams) {
        self.show_page_without_sidebar(&ui::spinner::reboot(params));
    }

    pub fn show_rescue(&self, params: &ConnectionParams) {
        self.show_page_without_sidebar(&update::rescue_flow(params.clone()));
    }

    pub fn connected_to_device(&self, tangara: Tangara) {
        let list = DeviceNavBuilder::new(tangara, self.controller.clone())
            .add_item(
                "Overview",
                "companion-overview-symbolic",
                move |device| ui::overview::page(device),
            )
            .add_item(
                "Lua Console",
                "companion-lua-console-symbolic",
                move |device| ui::lua::page(device),
            )
            .add_item(
                "Firmware Update",
                "companion-firmware-update-symbolic",
                |device| ui::update::flow(device),
            )
            .build();

        self.sidebar.device_nav.set_child(Some(&list));
    }

    pub fn device_error(&self, params: &ConnectionParams, error: &dyn Error, choice: SendOnce<DeviceErrorChoice>) {
        let message = format!("Error connecting to Tangara at {port_name}: {error}",
            port_name = params.serial.port_name);

        let retry_button = gtk::Button::builder()
            .label("Retry connection")
            .build();

        retry_button.connect_clicked({
            let choice = choice.clone();
            move |_| { choice.send(DeviceErrorChoice::Retry); }
        });

        let reboot_button = gtk::Button::builder()
            .label("Reboot Tangara")
            .build();

        reboot_button.connect_clicked({
            let choice = choice.clone();
            move |_| { choice.send(DeviceErrorChoice::Reboot); }
        });

        let reinstall_button = gtk::Button::builder()
            .label("Reinstall firmware")
            .build();

        reinstall_button.connect_clicked({
            let choice = choice.clone();
            move |_| { choice.send(DeviceErrorChoice::Reinstall); }
        });

        let buttons = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(10)
            .build();

        buttons.append(&retry_button);
        buttons.append(&reboot_button);
        buttons.append(&reinstall_button);

        let status_page = adw::StatusPage::builder()
            .icon_name("companion-computer-sadface-symbolic")
            .title("Tangara is not responding")
            .description(message)
            .child(&buttons)
            .build();

        let page = NavPageBuilder::clamped(&status_page)
            .title(status_page.title().as_str())
            .header(adw::HeaderBar::builder()
                .show_title(false)
                .build())
            .build();

        self.show_page_without_sidebar(&page);
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
    pub fn new(tangara: Tangara, controller: NavController) -> Self {
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

    pub fn lock(self: &Rc<Self>) -> DeviceNavLocked {
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
