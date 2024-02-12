//! This crate exists to bundle glib resources for tangara companion.
//! It only does anything in release builds. In debug builds, set
//! G_RESOURCE_OVERLAYS

#[cfg(not(debug_assertions))]
static DATA: &[u8] = include_bytes!(env!("GRESOURCE_PATH"));

#[cfg(not(debug_assertions))]
pub fn init() {
    let data = glib::Bytes::from_static(DATA);
    match gio::Resource::from_data(&data) {
        Ok(resource) => gio::resources_register(&resource),
        Err(error) => {
            eprintln!("resource::init: load resource error: {error}");
        }
    }
}

#[cfg(debug_assertions)]
pub fn init() {
    if std::env::var("G_RESOURCE_OVERLAYS").is_err() {
        panic!("G_RESOURCE_OVERLAYS env var must be set in debug builds!");
    }
}
