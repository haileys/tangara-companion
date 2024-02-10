extern crate walkdir;

use std::path::PathBuf;
use std::process::Command;

use walkdir::WalkDir;

fn main() {
    if !cfg!(debug_assertions) {
        build_resources();
    }
}

fn build_resources() {
    // make any changes to files in data/ cause a resource rebuild
    for entry in WalkDir::new(".") {
        println!("cargo:rerun-if-changed={}", entry.unwrap().path().display());
    }

    let cwd = std::env::current_dir().unwrap();
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let resource = out_dir.join("resource.gresource");

    let status = Command::new("glib-compile-resources")
        .current_dir(cwd)
        .arg("--target")
        .arg(&resource)
        .arg("resources.gresource.xml")
        .status()
        .unwrap();

    assert!(status.success());

    println!("cargo:rustc-env=GRESOURCE_PATH={}", resource.display());
}
