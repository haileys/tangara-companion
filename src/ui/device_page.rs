use adw::subclass::navigation_page::NavigationPageImpl;
use gtk::CompositeTemplate;
use gtk::glib::subclass::types::ObjectSubclass;
use gtk::subclass::prelude::*;

mod imp {

    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/zone/cooltech/tangara/companion/gtk/device_page.ui")]
    pub struct TangaraDevicePage {
        #[template_child]
        firmware_version_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TangaraDevicePage {
        const NAME: &'static str = "TangaraDevicePage";
        type ParentType = adw::NavigationPage;
        type Type = super::TangaraDevicePage;
    }

    impl NavigationPageImpl for TangaraDevicePage {}
    impl WidgetImpl for TangaraDevicePage {}
    impl ObjectImpl for TangaraDevicePage {}
}

glib::wrapper! {
    pub struct TangaraDevicePage(ObjectSubclass<imp::TangaraDevicePage>)
        @extends gtk::Widget, adw::NavigationPage;
}
