use std::rc::Rc;
use std::time::Duration;

use futures::pin_mut;
use futures::StreamExt;
use tangara_lib::device::{ConnectionParams, Tangara};

use crate::device::watch_port;
use crate::ui;
use crate::ui::nav::MainView;
use crate::ui::util::SendOnce;

const REBOOT_RECONNECT_ATTEMPTS: usize = 15;
const REBOOT_DELAY: Duration = Duration::from_secs(1);

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

    glib::spawn_future_local(watch_loop(view));

    window
}

async fn watch_loop(view: Rc<MainView>) {
    let watch = watch_port();
    pin_mut!(watch);

    view.show_welcome();

    loop {
        let Some(item) = watch.next().await else { break };

        if let Some(params) = item {
            found_device(view.clone(), params).await;
        } else {
            view.show_welcome();
        }
    }
}

async fn found_device(view: Rc<MainView>, params: ConnectionParams) {
    view.show_connecting(&params);

    'retry_connection: loop {
        let mut error_choice = try_connect(view.clone(), &params).await;

        'handle_error_choice: while let Some(choice) = error_choice {
            match choice {
                DeviceErrorChoice::Retry => {
                    continue 'retry_connection;
                }
                DeviceErrorChoice::Reboot => {
                    view.show_rebooting(&params);

                    match tangara_lib::device::reset(&params).await {
                        Ok(()) => {
                            for _ in 0..REBOOT_RECONNECT_ATTEMPTS {
                                glib::timeout_future(REBOOT_DELAY).await;
                                if let Ok(tangara) = Tangara::open(&params).await {
                                    view.connected_to_device(tangara);
                                }
                            }
                            continue 'retry_connection;
                        }
                        Err(error) => {
                            error_choice = SendOnce::with(|choice| {
                                view.device_error(&params, &error, choice);
                            }).await;
                            continue 'handle_error_choice;
                        }
                    }
                }
                DeviceErrorChoice::Reinstall => {
                    view.show_rescue(&params);
                    break;
                }
            }
        }

        break;
    }
}

async fn try_connect(view: Rc<MainView>, params: &ConnectionParams) -> Option<DeviceErrorChoice> {
    match Tangara::open(&params).await {
        Ok(tangara) => {
            view.connected_to_device(tangara);
            None
        }
        Err(error) => {
            let choice = SendOnce::with(|choice| {
                view.device_error(params, &error, choice)
            }).await;

            Some(choice.unwrap())
        }
    }
}

pub enum DeviceErrorChoice {
    Retry,
    Reboot,
    Reinstall,
}
