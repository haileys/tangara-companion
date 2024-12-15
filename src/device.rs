use std::time::Duration;

use futures::{future, SinkExt, Stream, StreamExt};
use futures::channel::mpsc;
use gtk::glib;
use tangara_lib::device::{ConnectionParams, Tangara};

const POLL_DURATION: Duration = Duration::from_secs(1);

fn watch_port() -> impl Stream<Item = Option<ConnectionParams>> {
    let (mut tx, rx) = mpsc::channel(1);

    glib::spawn_future_local(async move {
        let mut current = Tangara::find().ok();
        let _ = tx.send(current.clone()).await;

        while !tx.is_closed() {
            // TODO - see if we can subscribe to hardware events or something?
            glib::timeout_future(POLL_DURATION).await;

            let params = Tangara::find().ok();

            let current_port = current.as_ref().map(|p| &p.serial.port_name);
            let latest_port = params.as_ref().map(|p| &p.serial.port_name);

            if current_port == latest_port {
                continue;
            }

            current = params;
            let _: Result<_, _> = tx.send(current.clone()).await;
        }

        log::debug!("watch_port task finished");
    });

    rx
}

pub fn watch() -> impl Stream<Item = Option<Tangara>> {
    watch_port()
        .then(|params| async move {
            log::debug!("watch: new params: {params:?}");
            match params {
                Some(params) => Tangara::open(&params).await.map(Some),
                None => Ok(None),
            }
        })
        .filter_map(|result| future::ready(result
            .map_err(|error| { eprintln!("error opening tangara: {error:?}"); })
            .ok()))
}
