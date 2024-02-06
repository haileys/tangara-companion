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
pub struct DeviceViewContext {
    pub tangara: Rc<Tangara>,
    pub nav: Rc<ui::nav::DeviceNavController>,
}

impl Application {
    pub fn new(app: &adw::Application) -> Self {
        let view = ui::nav::MainView::new();

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .content(&*view)
            .width_request(400)
            .height_request(400)
            .default_width(800)
            .default_height(800)
            .build();

        Application {
            window,
            view,
        }
    }

    pub async fn set_tangara(&self, tangara: Option<Rc<Tangara>>) {
        self.view.set_device(tangara).await
    }
}
