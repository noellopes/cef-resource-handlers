use cef::{sys::cef_scheme_options_t, *};
use std::os::raw::c_int;

pub const APP_SCHEME: &str = "app";
pub const LOCAL_FILE_SCHEME: &str = "local";

pub fn register_local_file_scheme(registrar: Option<&mut SchemeRegistrar>) {
    let Some(registrar) = registrar else {
        eprintln!("Warning: Failed to register custom schemes, registrar is None.");
        return;
    };

    let options = cef_scheme_options_t::CEF_SCHEME_OPTION_STANDARD as c_int
        | cef_scheme_options_t::CEF_SCHEME_OPTION_SECURE as c_int
        | cef_scheme_options_t::CEF_SCHEME_OPTION_CORS_ENABLED as c_int;

    let local_file_scheme = CefString::from(LOCAL_FILE_SCHEME);
    if registrar.add_custom_scheme(Some(&local_file_scheme), options) == false as c_int {
        eprintln!("Warning: Failed to register {LOCAL_FILE_SCHEME} scheme.");
    }
}
