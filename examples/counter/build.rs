use anyhow::Result;
use examples_common::build_helpers::*;
use std::path::Path;

fn main() -> Result<()> {
    cargo_rerun_if_changed(Path::new("Cargo.toml"));

    #[cfg(target_os = "windows")]
    embed_windows_resources(Path::new("./resources/win/counter.rc"))?;

    let build_dir = build_directory()?;
    download_bootstrap_files(&build_dir)
}
