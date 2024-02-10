use std::rc::Rc;

use derive_more::Deref;

use crate::ui;
use crate::device::Tangara;

use super::nav::MainView;

#[derive(Deref)]
pub struct Application {
    #[deref]
    window: adw::ApplicationWindow,
    view: MainView,
}

#[derive(Clone)]
pub struct DeviceContext {
    pub tangara: Tangara,
    pub nav: Rc<ui::nav::DeviceNavController>,
}

impl Application {
    pub fn new(app: &adw::Application) -> Self {
        let style = gtk::CssProvider::new();
        style.load_from_resource("/zone/cooltech/tangara/companion/style/console.css");

        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().unwrap(),
            &style,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let view = ui::nav::MainView::new();

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .content(&*view)
            .width_request(400)
            .height_request(400)
            .default_width(800)
            .default_height(800)
            .build();

        if let Some(display) = gtk::gdk::Display::default() {
            let theme = gtk::IconTheme::for_display(&display);
            theme.add_resource_path("/zone/cooltech/tangara/companion/icons");
        }

        Application {
            window,
            view,
        }
    }

    pub async fn set_tangara(&self, tangara: Option<Tangara>) {
        self.view.set_device(tangara).await
    }
}
