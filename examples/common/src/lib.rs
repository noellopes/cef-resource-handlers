//! Shared infrastructure for the example applications.

pub mod app_handler;
pub mod default_app_delegates;
pub mod schemes;
pub mod startup;
pub mod web_page;

#[cfg(target_os = "macos")]
pub mod mac;
