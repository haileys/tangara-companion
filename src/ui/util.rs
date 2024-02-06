use glib::object::IsA;
use gtk::prelude::BoxExt;
use gtk::Align;

pub struct NavPageBuilder {
    view: adw::ToolbarView,
    title: String,
    header: Option<adw::HeaderBar>,
}

impl NavPageBuilder {
    pub fn clamped(object: &impl IsA<gtk::Widget>) -> Self {
        let view = adw::ToolbarView::builder()
            .content(&content_clamp(object))
            .build();

        Self { view, title: String::new(), header: None }
    }

    pub fn header(mut self, header: adw::HeaderBar) -> Self {
        self.header = Some(header);
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_owned();
        self
    }

    pub fn build(self) -> adw::NavigationPage {
        let header = self.header.unwrap_or_default();

        let view = self.view;
        view.add_top_bar(&header);

        adw::NavigationPage::builder()
            .child(&view)
            .title(self.title)
            .build()
    }
}

pub fn content_clamp(object: &impl IsA<gtk::Widget>) -> adw::Clamp {
    adw::Clamp::builder()
        .maximum_size(600)
        .child(object)
        .build()
}

pub fn spinner_content() -> gtk::Box {
    let box_ = gtk::Box::builder()
        .halign(Align::Center)
        .valign(Align::Center)
        .build();

    let spinner = gtk::Spinner::builder()
        .spinning(true)
        .build();

    box_.append(&spinner);

    box_
}
