#[cfg(target_os = "windows")]
mod win;

mod bootstrap;
mod build_support;
mod download;

#[cfg(target_os = "windows")]
pub use win::embed_windows_resources;

pub use bootstrap::download_bootstrap_files;
pub use build_support::{build_directory, cargo_rerun_if_changed};
