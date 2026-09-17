use std::path::Path;

pub fn cargo_rerun_if_changed(path: &Path) {
    println!("cargo:rerun-if-changed={path:?}");
}
