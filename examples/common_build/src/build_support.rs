use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub fn cargo_rerun_if_changed(path: &Path) {
    println!("cargo:rerun-if-changed={path:?}");
}

pub fn build_directory() -> Result<PathBuf> {
    let out_dir = std::env::var("OUT_DIR").context("Failed to get OUT_DIR")?;
    let out_path = Path::new(&out_dir);

    for ancestor in out_path.ancestors() {
        if ancestor.file_name() == Some(std::ffi::OsStr::new("build")) {
            return ancestor
                .parent()
                .map(Path::to_path_buf)
                .context("Could not determine profile directory from OUT_DIR");
        }
    }

    anyhow::bail!("Could not find 'build' component in OUT_DIR: {out_dir}")
}
