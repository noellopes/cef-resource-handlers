#[cfg(target_os = "windows")]
mod win;

mod build_support;

#[cfg(target_os = "windows")]
pub use win::embed_windows_resources;

pub use build_support::cargo_rerun_if_changed;
