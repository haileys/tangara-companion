use std::time::Duration;

use futures::Stream;
use gtk::glib;
use tangara_lib::device::{ConnectionParams, FindTangaraError, Tangara};

const POLL_DURATION: Duration = Duration::from_secs(1);

pub fn watch_port() -> impl Stream<Item = Option<ConnectionParams>> {
    async_stream::stream! {
        let mut current = find_device();
        yield current.clone();

        loop {
            // TODO - see if we can subscribe to hardware events or something?
            glib::timeout_future(POLL_DURATION).await;

            let params = find_device();

            if params != current {
                current = params;
                yield current.clone();
            }
        }
    }
}

fn find_device() -> Option<ConnectionParams> {
    match Tangara::find() {
        Ok(params) => Some(params),
        Err(FindTangaraError::NoTangara) => None,
        Err(err) => {
            log::warn!("find_device: {err}");
            None
        }
    }
}
