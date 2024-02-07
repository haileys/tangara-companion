use adw::prelude::{PreferencesPageExt, PreferencesGroupExt};
use gtk::pango::EllipsizeMode;
use gtk::prelude::{IsA, BoxExt, RangeExt, WidgetExt, ScaleExt};
use gtk::Orientation;

use crate::settings;
use crate::settings::{EnumSetting, IntRangeSetting};
use crate::ui::application::DeviceContext;
use crate::ui::util::NavPageBuilder;

pub fn page(device: DeviceContext) -> adw::NavigationPage {
    let pref = PageBuilder::new(
        adw::PreferencesPage::builder()
            .title("Settings")
            .build());

    pref.group("Headphones")
        .row("Maximum volume limit", combo::<settings::audio::MaximumVolumeLimit>())
        .row("Volume", range::<settings::audio::Volume>())
        .row("Balance", range::<settings::audio::Balance>())
        .build();

    pref.group("Display")
        .row("Brightness", range::<settings::display::Brightness>())
        .build();

    pref.group("Input")
        .row("Input Method", combo::<settings::input::InputMethod>())
        .build();

    NavPageBuilder::clamped(&pref.page)
        .title(pref.page.title().as_str())
        .build()
}

fn combo<T: EnumSetting>() -> gtk::DropDown {
    let list = gtk::StringList::default();

    for item in T::ITEMS {
        list.append(&item.to_string());
    }

    let value = T::default(); // TODO get this from device

    let selected = T::ITEMS.iter()
        .position(|item| item == &value)
        .and_then(|idx| u32::try_from(idx).ok())
        .unwrap_or(0);

    gtk::DropDown::builder()
        .model(&list)
        .selected(selected)
        .build()
}

fn range<T: IntRangeSetting>() -> gtk::Scale {
    let value = T::default(); // TODO get this from device

    let scale = gtk::Scale::with_range(
        Orientation::Horizontal,
        T::MIN as f64,
        T::MAX as f64,
        1.0,
    );

    scale.set_value(value.into() as f64);

    for (notch, label) in T::NOTCHES {
        scale.add_mark(*notch as f64, gtk::PositionType::Bottom, *label);
    }

    scale
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
