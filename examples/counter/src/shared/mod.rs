use cef::*;
use examples_common::startup::initialize_cef;

pub(crate) mod counter_app;
pub(crate) mod counter_handler;
pub(crate) mod counter_schemes;
pub(crate) mod counter_web_page_handler;

#[cfg(target_os = "macos")]
pub(crate) type Library = library_loader::LibraryLoader;

#[cfg(not(target_os = "macos"))]
pub(crate) type Library = ();

pub(crate) fn load_cef() -> Result<Library, anyhow::Error> {
    #[cfg(target_os = "macos")]
    let library = load_macos_library()?;
    #[cfg(not(target_os = "macos"))]
    let library = ();

    // Initialize the CEF API version.
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);

    #[cfg(target_os = "macos")]
    crate::mac::setup_counter_application();

    Ok(library)
}

#[cfg(target_os = "macos")]
fn load_macos_library() -> anyhow::Result<Library> {
    let loader = library_loader::LibraryLoader::new(&std::env::current_exe()?, false);

    match loader.load() {
        true => Ok(loader),
        false => Err(anyhow!("Failed to load library")),
    }
}

pub(crate) fn run_main(
    main_args: &MainArgs,
    cmd_line: &CommandLine,
    sandbox_info: *mut u8,
) -> Result<(), anyhow::Error> {
    let switch = CefString::from("type");
    let is_browser_process = cmd_line.has_switch(Some(&switch)) != 1;
    let mut app = counter_app::CounterApp::new();

    let browser_process_type = if is_browser_process {
        "browser process".into()
    } else {
        let process_type = CefString::from(&cmd_line.switch_value(Some(&switch)));
        format!("non-browser process {process_type}")
    };

    println!("launch {browser_process_type}");

    let ret = execute_process(Some(main_args), Some(&mut app), sandbox_info);

    if (is_browser_process && ret != -1) || (!is_browser_process && ret < 0) {
        anyhow::bail!("Cannot execute {browser_process_type}, return code: {ret}");
    }

    if is_browser_process {
        initialize_cef(main_args, &mut app, sandbox_info)?;

        #[cfg(target_os = "macos")]
        let _delegate = crate::mac::setup_counter_app_delegate();

        run_message_loop();

        shutdown();
    }

    Ok(())
}
