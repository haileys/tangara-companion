use std::fmt::Debug;
use std::time::Duration;

use adw::prelude::{PreferencesPageExt, PreferencesGroupExt};
use gtk::pango::EllipsizeMode;
use gtk::prelude::{IsA, BoxExt, RangeExt, WidgetExt, ScaleExt};
use gtk::Orientation;
use tangara_lib::device::Tangara;

use crate::settings::{self, FromLuaOutput, Setting, ToLuaExpr};
use crate::settings::{EnumSetting, IntRangeSetting};
use crate::ui::application::DeviceContext;
use crate::ui::util::NavPageBuilder;
use crate::util::watch1;

pub fn page(device: DeviceContext) -> adw::NavigationPage {
    let tangara = &device.tangara;

    let pref = PageBuilder::new(
        adw::PreferencesPage::builder()
            .title("Settings")
            .build());

    pref.group("Headphones")
        .row("Maximum volume limit", combo::<settings::audio::MaximumVolumeLimit>(tangara))
        // .row("Volume", range::<settings::audio::Volume>())
        .row("Balance", range::<settings::audio::Balance>(tangara))
        .build();

    pref.group("Display")
        .row("Brightness", range::<settings::display::Brightness>(tangara))
        .build();

    pref.group("Input")
        .row("Input Method", combo::<settings::input::InputMethod>(tangara))
        .build();

    NavPageBuilder::clamped(&pref.page)
        .title(pref.page.title().as_str())
        .build()
}

fn combo<T: EnumSetting + Debug + Clone>(tangara: &Tangara) -> gtk::DropDown {
    let list = gtk::StringList::default();

    for item in T::ITEMS {
        list.append(&item.to_string());
    }

    let control = gtk::DropDown::builder()
        .model(&list)
        .sensitive(false)
        .build();

    glib::spawn_future_local({
        let conn = tangara.connection().clone();
        let control = control.clone();
        async move {
            let value = T::PROPERTY.get(&conn).await.unwrap();

            let selected = T::ITEMS.iter()
                .position(|item| item == &value)
                .and_then(|idx| u32::try_from(idx).ok())
                .unwrap_or(0);

            control.set_selected(selected);
            control.set_sensitive(true);
        }
    });

    control.connect_selected_item_notify({
        let tx = setting_sender(tangara);
        move |control| {
            if let Some(value) = T::ITEMS.get(control.selected() as usize).cloned() {
                tx.send(Some(value)).ok();
            }
        }
    });

    control
}

fn range<T: IntRangeSetting + Debug>(tangara: &Tangara) -> gtk::Scale {
    let scale = gtk::Scale::with_range(
        Orientation::Horizontal,
        T::MIN as f64,
        T::MAX as f64,
        1.0,
    );

    for (notch, label) in T::NOTCHES {
        scale.add_mark(*notch as f64, gtk::PositionType::Bottom, *label);
    }

    scale.set_sensitive(false);

    glib::spawn_future_local({
        let scale = scale.clone();
        let conn = tangara.connection().clone();
        async move {
            let value = T::PROPERTY.get(&conn).await.unwrap();
            let value = value.into();
            scale.set_value(value as f64);
            scale.set_sensitive(true);
        }
    });

    scale.connect_change_value({
        let tx = setting_sender(tangara);
        move |_, _, value| {
            let value = value as i32;
            let value = T::from(value);
            tx.send(Some(value)).ok();
            glib::Propagation::Proceed
        }
    });

    scale
}

fn setting_sender<T: Setting + Debug>(tangara: &Tangara) -> watch1::Sender<Option<T>> {
    let (tx, mut rx) = watch1::channel(None);
    let connection = tangara.connection().clone();

    glib::spawn_future_local(async move {
        while let Some(value) = rx.recv().await {
            let Some(value) = value else { continue };

            log::info!("Setting property {}.{} to: {value:?}", T::PROPERTY.module, T::PROPERTY.property);

            // try three times to set the value with a little delay each time
            for _ in 1..3 {
                match T::PROPERTY.set(&connection, &value).await {
                    Ok(()) => { break }
                    Err(error) => {
                        log::warn!("error setting property: {error:?}");
                        glib::timeout_future(Duration::from_millis(100)).await;
                    }
                }
            }
        }
    });

    tx
}

struct PageBuilder {
    page: adw::PreferencesPage,
    label_group: gtk::SizeGroup,
}

impl PageBuilder {
    pub fn new(page: adw::PreferencesPage) -> Self {
        let label_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
        PageBuilder { page, label_group }
    }

    pub fn group(&self, title: &str) -> GroupBuilder {
        let group = adw::PreferencesGroup::builder()
            .title(title)
            .build();

        GroupBuilder {
            page: self.page.clone(),
            label_group: self.label_group.clone(),
            group,
        }
    }
}

struct GroupBuilder {
    page: adw::PreferencesPage,
    label_group: gtk::SizeGroup,
    group: adw::PreferencesGroup,
}

impl GroupBuilder {
    pub fn row(self, label: &str, widget: impl IsA<gtk::Widget>) -> Self {
        let label = gtk::Label::builder()
            .label(label)
            .ellipsize(EllipsizeMode::End)
            .xalign(0.0)
            .halign(gtk::Align::Fill)
            .justify(gtk::Justification::Left)
            .build();

        widget.set_hexpand(true);
        widget.set_hexpand_set(true);

        let box_ = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .spacing(12)
            .build();

        box_.append(&label);
        box_.append(&widget);

        let row = adw::PreferencesRow::builder()
            .child(&box_)
            .build();

        self.group.add(&row);
        self.label_group.add_widget(&label);

        self
    }

    pub fn build(self) {
        self.page.add(&self.group);
    }
}
