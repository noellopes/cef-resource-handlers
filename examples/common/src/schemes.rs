use cef::*;

pub const APP_SCHEME: &str = "app";
pub const LOCAL_FILE_SCHEME: &str = "local";

pub fn register_local_file_scheme(registrar: Option<&mut SchemeRegistrar>) {
    let Some(registrar) = registrar else {
        eprintln!("Warning: Failed to register custom schemes, registrar is None.");
        return;
    };

    let options = sys::cef_scheme_options_t::CEF_SCHEME_OPTION_STANDARD as i32
        | sys::cef_scheme_options_t::CEF_SCHEME_OPTION_SECURE as i32
        | sys::cef_scheme_options_t::CEF_SCHEME_OPTION_CORS_ENABLED as i32;

    let local_file_scheme = CefString::from(LOCAL_FILE_SCHEME);
    if registrar.add_custom_scheme(Some(&local_file_scheme), options) == false as i32 {
        eprintln!("Warning: Failed to register {LOCAL_FILE_SCHEME} scheme.");
    }
}
