use crate::shared;
use cef::*;
use common::startup::{load_cef, run_main};

#[unsafe(no_mangle)]
unsafe extern "C" fn RunWinMain(
    instance: sys::HINSTANCE,
    _command_line: *const u8,
    _command_show: i32,
    sandbox_info: *mut u8,
) -> i32 {
    let Ok(_library) = load_cef() else {
        return 1;
    };

    let main_args = MainArgs { instance };
    let args = args::Args::from(main_args);
    let Some(cmd_line) = args.as_cmd_line() else {
        return 1;
    };

    let mut app = shared::hello_app::HelloApp::new();
    match run_main(&mut app, args.as_main_args(), &cmd_line, sandbox_info) {
        Err(_) => 1,
        _ => 0,
    }
}
