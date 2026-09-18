//! Shared infrastructure for the example applications.

pub mod app_handler;
pub mod build_helpers;
pub mod startup;

#[cfg(target_os = "macos")]
pub mod mac;
