use adw::prelude::{ActionRowExt, NavigationPageExt, PreferencesGroupExt, PreferencesPageExt};
use derive_more::Deref;
use glib::object::Cast;
use glib::types::StaticType;
use gtk::{FileDialog, FileFilter};
use gtk::gio::{Cancellable, File};
use gtk::prelude::WidgetExt;

#[derive(Deref)]
pub struct UpdateFlow {
    #[deref]
    page: adw::NavigationPage,
    nav: adw::NavigationView,
}

impl UpdateFlow {
    pub fn new() -> Self {
        let nav = adw::NavigationView::new();
        nav.add(&*SelectFirmwarePage::new());

        let page = adw::NavigationPage::builder()
            .child(&nav)
            .build();

        UpdateFlow { page, nav }
    }
}

#[derive(Deref)]
pub struct SelectFirmwarePage {
    #[deref]
    page: adw::NavigationPage,
}

impl SelectFirmwarePage {
    pub fn new() -> Self {
        let pref_page = adw::PreferencesPage::builder()
            .title("Update Firmware")
            .build();

        pref_page.add(&select_group());

        let header = adw::HeaderBar::new();

        let view = adw::ToolbarView::builder()
            .content(&pref_page)
            .build();

        view.add_top_bar(&header);

        let page = adw::NavigationPage::builder()
            .title(pref_page.title())
            .tag("select-firmware")
            .child(&view)
            .build();

        SelectFirmwarePage { page }
    }
}

fn select_group() -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::new();

    let select_firmware = adw::ActionRow::builder()
        .title("Select firmware...")
        .activatable(true)
        .build();

    select_firmware.add_suffix(&gtk::Image::builder()
        .icon_name("folder-symbolic")
        .build());

    select_firmware.connect_activated(|widget| {
        let filter = FileFilter::new();
        filter.add_suffix("tra");

        let current_dir = std::env::current_dir().unwrap();

        let window: Option<gtk::Window> = widget
            .ancestor(gtk::Window::static_type())
            .and_then(|ancestor| ancestor.dynamic_cast().ok());

        FileDialog::builder()
            .default_filter(&filter)
            .initial_folder(&File::for_path(&current_dir))
            .title("Select Tangara firmware")
            .modal(true)
            .build()
            .open(window.as_ref(), Cancellable::NONE, {
                move |result| {
                    eprintln!("{result:?}");
                }
                // let app = app.clone();
                // move |result| {
                //     match result {
                //         Ok(file) => {
                //             let Some(path) = file.path() else {
                //                 // no path?
                //                 eprintln!("no path in file from file dialog");
                //                 return;
                //             };

                //             match Firmware::open(&path) {
                //                 Ok(firmware) => {
                //                     app.nav.push(&firmware_page(app.clone(), FirmwarePage {
                //                         firmware: Arc::new(firmware),
                //                     }));
                //                 }
                //                 Err(error) => {
                //                     eprintln!("read firmware error: {}", error);
                //                 }
                //             }
                //         }
                //         Err(error) => {
                //             // TODO how do we surface this to user?
                //             eprintln!("file dialoag error: {error:?}");
                //         }
                //     }
                // }
            });
    });

    group.add(&select_firmware);

    group
}
