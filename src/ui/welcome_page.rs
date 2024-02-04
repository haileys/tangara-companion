use adw::subclass::navigation_page::NavigationPageImpl;
use gtk::CompositeTemplate;
use gtk::glib::subclass::types::ObjectSubclass;
use gtk::subclass::prelude::*;

mod imp {
    use glib::subclass;

    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/zone/cooltech/tangara/companion/gtk/welcome_page.ui")]
    pub struct TngWelcomePage;

    #[glib::object_subclass]
    impl ObjectSubclass for TngWelcomePage {
        const NAME: &'static str = "TngWelcomePage";
        type ParentType = adw::NavigationPage;
        type Type = super::TngWelcomePage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl NavigationPageImpl for TngWelcomePage {}
    impl WidgetImpl for TngWelcomePage {}
    impl ObjectImpl for TngWelcomePage {}
}

glib::wrapper! {
    pub struct TngWelcomePage(ObjectSubclass<imp::TngWelcomePage>)
        @extends gtk::Widget, adw::NavigationPage;
}

impl TngWelcomePage {
    pub fn new() -> Self {
        glib::Object::builder::<Self>().build()
    }
}
