use super::counter_schemes::*;
use super::counter_web_page_handler::*;
use cef::*;
use cef_dll_sys::*;
use cef_resource_handlers::*;
use common::app_handler::*;
use common::default_app_delegates::{DefaultAppBrowserViewDelegate, DefaultAppWindowDelegate};
use std::cell::RefCell;
use std::os::raw::c_int;
use std::sync::{Arc, atomic::AtomicI32};

wrap_app! {
    pub(crate) struct CounterApp;

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(CounterBrowserProcessHandler::new(RefCell::new(None)))
        }

        fn on_register_custom_schemes(&self, registrar: Option<&mut SchemeRegistrar>) {
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
    }
}

wrap_browser_process_handler! {
    struct CounterBrowserProcessHandler {
        client: RefCell<Option<Client>>,
    }

    impl BrowserProcessHandler {
        // The real lifespan of cef starts from `on_context_initialized`, so all the cef objects should be manipulated after that.
        fn on_context_initialized(&self) {
            debug_assert_ne!(currently_on(ThreadId::UI), 0);

            // Check if Alloy style will be used.
            let command_line = command_line_get_global().expect("Failed to get command line");
            let use_alloy_style =
                command_line.has_switch(Some(&CefString::from("use-alloy-style"))) != 0;
            let runtime_style = if use_alloy_style {
                RuntimeStyle::ALLOY
            } else {
                RuntimeStyle::DEFAULT
            };

            {
                // AppHandler implements browser-level callbacks.
                let mut client = self.client.borrow_mut();
                *client = Some(AppHandlerClient::new(AppHandler::new(use_alloy_style)));
            }

            // Specify CEF browser settings here.
            let settings = BrowserSettings::default();

            let url = CefString::from(WebPage::Home.url().as_str());

            // Register the APP_SCHEME handler
            if let Err(error) =
                WebPageResourceHandlerFactory::<CounterWebPageHandler>::register_with_context(
                    APP_SCHEME,
                    None,
                    Arc::new(AtomicI32::new(0)),
                )
            {
                eprintln!("Warning: {error}");
            }

            // Register the LOCAL_FILE_SCHEME handler
            if let Err(error) = LocalFileResourceHandlerFactory::register(LOCAL_FILE_SCHEME, None) {
                eprintln!("Warning: {error}");
            }

            // Views is enabled by default (add `--use-native` to disable).
            let use_views = command_line.has_switch(Some(&CefString::from("use-native"))) != 0;

            // If using Views create the browser using the Views framework, otherwise
            // create the browser using the native platform framework.
            if use_views {
                // Create the BrowserView.
                let mut client = self.default_client();
                let mut delegate = DefaultAppBrowserViewDelegate::new(runtime_style);
                let browser_view = browser_view_create(
                    client.as_mut(),
                    Some(&url),
                    Some(&settings),
                    None,
                    None,
                    Some(&mut delegate),
                );

                // Configure the initial show state.
                let initial_show_state_switch = Some(&CefString::from("initial-show-state"));
                let initial_show_state = match command_line.has_switch(initial_show_state_switch) {
                    0 => ShowState::NORMAL,
                    _ => {
                        let value =
                            CefString::from(&command_line.switch_value(initial_show_state_switch))
                                .to_string();
                        match value.as_str() {
                            "minimized" => ShowState::MINIMIZED,
                            "maximized" => ShowState::MAXIMIZED,
                            // Hidden show state is only supported on MacOS.
                            #[cfg(target_os = "macos")]
                            "hidden" => ShowState::HIDDEN,
                            _ => ShowState::NORMAL,
                        }
                    }
                };

                // Create the Window. It will show itself after creation.
                let mut delegate = DefaultAppWindowDelegate::new(
                    RefCell::new(browser_view),
                    runtime_style,
                    initial_show_state,
                );
                window_create_top_level(Some(&mut delegate));
            } else {
                // Information used when creating the native window.
                let window_info = WindowInfo {
                    runtime_style,
                    ..Default::default()
                };

                #[cfg(target_os = "windows")]
                let window_info = window_info.set_as_popup(Default::default(), "counter");

                let mut client = self.default_client();
                browser_host_create_browser(
                    Some(&window_info),
                    client.as_mut(),
                    Some(&url),
                    Some(&settings),
                    None,
                    None,
                );
            }
        }

        fn default_client(&self) -> Option<Client> {
            self.client.borrow().clone()
        }
    }
}
