use std::rc::Rc;

use futures::StreamExt;
use tangara_lib::device::{ConnectionParams, Tangara};

use crate::ui;
use crate::device;
use crate::ui::nav::MainView;

#[derive(Clone)]
pub struct DeviceContext {
    pub tangara: Tangara,
    pub nav: Rc<ui::nav::DeviceNavController>,
}

pub fn run_application(app: &adw::Application) -> adw::ApplicationWindow {
    let style = gtk::CssProvider::new();
    style.load_from_resource("/zone/cooltech/tangara/Companion/style/console.css");

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().unwrap(),
        &style,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let view = Rc::new(ui::nav::MainView::new());

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .content(&**view)
        .width_request(400)
        .height_request(400)
        .default_width(800)
        .default_height(800)
        .build();

    if let Some(display) = gtk::gdk::Display::default() {
        let theme = gtk::IconTheme::for_display(&display);
        theme.add_resource_path("/zone/cooltech/tangara/Companion/icons");
    }

    glib::spawn_future_local(watch_port(view));

    window
}

async fn watch_port(view: Rc<MainView>) {
    let mut watch = Box::pin(device::watch_port());

    while let Some(params) = watch.next().await {
        let Some(params) = params else {
            view.show_welcome();
            continue;
        };

        if try_connect(view.clone(), params).await.is_err() {
            break;
        }
    }
}

async fn try_connect(view: Rc<MainView>, params: ConnectionParams) -> Result<(), ()> {
    view.show_connecting(&params);

    match Tangara::open(&params).await {
        Ok(tangara) => {
            view.connected_to_device(tangara);
            Ok(())
        }
        Err(error) => {
            let message = error.to_string();
            let params = params.clone();
            show_device_error(view, &message, params);
            Err(())
        }
    }
}

fn show_device_error(view: Rc<MainView>, message: &str, params: ConnectionParams) {
    view.device_error(&message, {
        let view = view.clone();
        move || {
            view.show_connecting(&params);
            glib::spawn_future_local(async move {
                match tangara_lib::device::reset(&params).await {
                    Ok(()) => {
                        let _ = try_connect(view.clone(), params).await;
                    }
                    Err(err) => {
                        let message = err.to_string();
                        show_device_error(view.clone(), &message, params);
                    }
                }
            });
        }
    });
}
