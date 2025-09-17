use adw::prelude::ActionRowExt;
use gtk::pango::EllipsizeMode;
use gtk::Align;

use derive_more::Deref;

#[derive(Deref)]
pub struct LabelRow {
    #[deref]
    row: adw::ActionRow,
}

impl LabelRow {
    pub fn new(title: &str, value: &str) -> Self {
        let label = gtk::Label::builder()
            .valign(Align::Center)
            .ellipsize(EllipsizeMode::End)
            .css_classes(["dim-label"])
            .label(value)
            .selectable(true)
            .build();

        let row = adw::ActionRow::builder()
            .title(title)
            .build();

        row.add_suffix(&label);

        LabelRow { row }
    }
}
