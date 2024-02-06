use adw::prelude::{NavigationPageExt, PreferencesGroupExt, PreferencesPageExt};
use gtk::ContentFit;

use derive_more::Deref;

use crate::device::Tangara;
use crate::device::info;
use crate::ui::application::DeviceContext;

use super::label_row::LabelRow;

#[derive(Deref)]
pub struct OverviewPage {
    #[deref]
    page: adw::NavigationPage,
}

impl OverviewPage {
    pub fn new(device: DeviceContext, info: &info::Info) -> Self {
        let title_group = adw::PreferencesGroup::builder()
            .build();

        let title_picture = gtk::Picture::for_resource("/zone/cooltech/tangara/companion/assets/logo.svg");
        title_picture.set_can_shrink(false);
        title_picture.set_content_fit(ContentFit::ScaleDown);
        title_group.add(&title_picture);

        let device_group = device_group(&device.tangara);
        let firmware_group = firmware_group(&info.firmware);
        let database_group = database_group(&info.database);

        let pref_page = adw::PreferencesPage::builder()
            .title("Device Information")
            .build();

        pref_page.add(&title_group);
        pref_page.add(&device_group);
        pref_page.add(&firmware_group);
        pref_page.add(&database_group);

        let header = adw::HeaderBar::new();

        let view = adw::ToolbarView::builder()
            .content(&pref_page)
            .build();

        view.add_top_bar(&header);

        let page = adw::NavigationPage::builder()
            .title(pref_page.title())
            .build();

        page.set_child(Some(&view));

        OverviewPage { page }
    }
}

fn device_group(tangara: &Tangara) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .build();

    let port = LabelRow::new("Serial port", tangara.serial_port_name());
    group.add(&*port);

    group
}

fn firmware_group(firmware: &info::Firmware) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Firmware")
        .build();

    let version = LabelRow::new("Version", &firmware.version);
    group.add(&*version);

    let samd = LabelRow::new("SAMD", &firmware.samd);
    group.add(&*samd);

    let collation = LabelRow::new("Collation", &firmware.collation);
    group.add(&*collation);

    group
}

fn database_group(database: &info::Database) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Database")
        .build();

    let schema = LabelRow::new("Schema version", &database.schema_version);
    group.add(&*schema);

    let disk_size = database.disk_size.map(render_size);
    let disk_size = disk_size.as_deref().unwrap_or("unknown");
    let size = LabelRow::new("Size on disk", disk_size);
    group.add(&*size);

    group
}

fn render_size(bytes: u64) -> String {
    if bytes < 1024 { return format!("{bytes} b") }

    let kib = bytes / 1024;
    if kib < 1024 { return format!("{kib} KiB") }

    let mib = bytes / 1024;
    if mib < 1024 { return format!("{mib} MiB") }

    let gib = bytes / 1024;
    format!("{gib} GiB")
}
