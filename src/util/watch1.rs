//! An implementation of a watch channel but for one sender and one receiver

use std::sync::{Arc, Mutex};

pub struct Sender<T> {
    tx: async_watch::Sender<Arc<Mutex<Option<T>>>>,
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), T> {
        self.tx
            .send(Arc::new(Mutex::new(Some(value))))
            .map_err(|err| take(err.value()))
    }
}

pub struct Receiver<T> {
    rx: async_watch::Receiver<Arc<Mutex<Option<T>>>>,
}

impl<T> Receiver<T> {
    pub async fn recv(&mut self) -> Option<T> {
        self.rx.recv().await.ok().map(take)
    }
}

pub fn channel<T>(initial: T) -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = async_watch::channel(Arc::new(Mutex::new(Some(initial))));
    (Sender { tx }, Receiver { rx })
}

fn take<T>(value: Arc<Mutex<Option<T>>>) -> T {
    value.lock().unwrap().take().unwrap()
}
