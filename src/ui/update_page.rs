use adw::prelude::{NavigationPageExt, PreferencesGroupExt, PreferencesPageExt};

use derive_more::Deref;

#[derive(Deref)]
pub struct UpdatePage {
    #[deref]
    page: adw::NavigationPage,
}

impl UpdatePage {
    pub fn new() -> Self {
        let select_firmware = adw::ActionRow::builder()
            .title("Select firmware...")
            .child(&gtk::Image::builder()
                .icon_name("folder-symbolic")
                .build())
            .build();

        let select_group = adw::PreferencesGroup::builder()
            .build();

        select_group.add(&select_firmware);

        let pref_page = adw::PreferencesPage::builder()
            .title("Update Firmware")
            .build();

        pref_page.add(&select_group);

        let header = adw::HeaderBar::new();

        let view = adw::ToolbarView::builder()
            .content(&pref_page)
            .build();

        view.add_top_bar(&header);

        let page = adw::NavigationPage::builder()
            .title(pref_page.title())
            .build();

        page.set_child(Some(&view));

        UpdatePage { page }
    }
}
