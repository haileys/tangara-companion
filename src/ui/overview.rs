use adw::prelude::{NavigationPageExt, PreferencesGroupExt, PreferencesPageExt};

use thiserror::Error;

use crate::device;
use crate::device::Tangara;
use crate::device::info;
use crate::ui;
use crate::ui::application::DeviceContext;
use crate::ui::label_row::LabelRow;
use crate::ui::util::spinner_content;

pub fn page(device: DeviceContext) -> adw::NavigationPage {
    let header = adw::HeaderBar::new();

    let view = adw::ToolbarView::builder()
        .content(&spinner_content())
        .build();

    view.add_top_bar(&header);

    let page = adw::NavigationPage::builder()
        .title("Overview")
        .build();

    page.set_child(Some(&view));

    glib::spawn_future_local(async move {
        match fetch_info(&device.tangara).await {
            Ok(info) => {
                let content = show_info(device.clone(), &info);
                view.set_content(Some(&content));
            }
            Err(error) => {
                let content = adw::StatusPage::builder()
                    .icon_name("computer-fail-symbolic")
                    .title("Error communicating with Tangara")
                    .description(format!("{error}"))
                    .build();

                view.set_content(Some(&content));
            }
        }
    });

    page
}

#[derive(Debug, Error)]
enum FetchInfoError {
    #[error("connecting to device: {0}")]
    Open(#[from] device::connection::OpenError),
    #[error("querying device info: {0}")]
    Query(#[from] device::info::InfoError),
}

async fn fetch_info(tangara: &Tangara) -> Result<device::info::Info, FetchInfoError> {
    let info = device::info::get(tangara.connection()).await?;
    Ok(info)
}

fn show_info(device: DeviceContext, info: &device::info::Info) -> adw::PreferencesPage {
    let title_group = adw::PreferencesGroup::builder()
        .build();

    let title_logo = ui::widgets::logo::logo();
    title_logo.set_can_shrink(false);
    title_group.add(&title_logo);

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

    pref_page
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
