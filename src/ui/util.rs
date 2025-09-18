use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;

use futures::channel::oneshot;
use futures::FutureExt;
use glib::object::IsA;
use gtk::prelude::BoxExt;
use gtk::Align;

pub struct NavPageBuilder {
    view: adw::ToolbarView,
    title: String,
    header: Option<adw::HeaderBar>,
}

impl NavPageBuilder {
    pub fn new(object: &impl IsA<gtk::Widget>) -> Self {
        let view = adw::ToolbarView::builder()
            .content(object)
            .build();

        Self { view, title: String::new(), header: None }
    }

    pub fn clamped(object: &impl IsA<gtk::Widget>) -> Self {
        Self::new(&content_clamp(object))
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

pub struct SendOnce<T> {
    tx: Rc<RefCell<Option<oneshot::Sender<T>>>>,
}

impl<T> SendOnce<T> {
    pub async fn with(func: impl FnOnce(Self)) -> Option<T> {
        let (tx, rx) = Self::channel();
        func(tx);
        rx.await
    }

    pub fn channel() -> (SendOnce<T>, impl Future<Output = Option<T>>) {
        let (tx, rx) = oneshot::channel();
        let tx = Self::new(tx);
        let rx = rx.map(|result| result.ok());
        (tx, rx)
    }

    pub fn new(tx: oneshot::Sender<T>) -> Self {
        SendOnce { tx: Rc::new(RefCell::new(Some(tx))) }
    }

    pub fn send(&self, value: T) {
        if let Some(tx) = self.tx.borrow_mut().take() {
            let _ = tx.send(value);
        }
    }
}

impl<T> Clone for SendOnce<T> {
    fn clone(&self) -> Self {
        SendOnce { tx: self.tx.clone() }
    }
}
