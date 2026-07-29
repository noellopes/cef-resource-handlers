use super::hello_handler::*;
use super::hello_schemes::*;
use super::hello_web_page_handler::*;
use cef::*;
use cef_dll_sys::*;
use cef_resource_handlers::*;
use std::cell::RefCell;
use std::os::raw::c_int;

wrap_window_delegate! {
    struct HelloWindowDelegate {
        browser_view: RefCell<Option<BrowserView>>,
        runtime_style: RuntimeStyle,
        initial_show_state: ShowState,
    }

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size {
                width: 800,
                height: 600,
            }
        }
    }

    impl PanelDelegate {}

    impl WindowDelegate {
        fn on_window_created(&self, window: Option<&mut Window>) {
            // Add the browser view and show the window.
            let browser_view = self.browser_view.borrow();
            let (Some(window), Some(browser_view)) = (window, browser_view.as_ref()) else {
                return;
            };
            let mut view = View::from(browser_view);
            window.add_child_view(Some(&mut view));

            if self.initial_show_state != ShowState::HIDDEN {
                window.show();
            }
        }

        fn on_window_destroyed(&self, _window: Option<&mut Window>) {
            let mut browser_view = self.browser_view.borrow_mut();
            *browser_view = None;
        }

        fn can_close(&self, _window: Option<&mut Window>) -> i32 {
            // Allow the window to close if the browser says it's OK.
            let browser_view = self.browser_view.borrow();
            let browser_view = browser_view.as_ref().expect("BrowserView is None");
            if let Some(browser) = browser_view.browser() {
                let browser_host = browser.host().expect("BrowserHost is None");
                browser_host.try_close_browser()
            } else {
                1
            }
        }

        fn initial_show_state(&self, _window: Option<&mut Window>) -> ShowState {
            self.initial_show_state
        }

        fn window_runtime_style(&self) -> RuntimeStyle {
            self.runtime_style
        }

        fn can_resize(&self, _window: Option<&mut Window>) -> c_int {
            true as _
        }

        fn can_maximize(&self, _window: Option<&mut Window>) -> c_int {
            true as _
        }

        fn can_minimize(&self, _window: Option<&mut Window>) -> c_int {
            true as _
        }
    }
}

wrap_browser_view_delegate! {
    struct HelloBrowserViewDelegate {
        runtime_style: RuntimeStyle,
    }

    impl ViewDelegate {}

    impl BrowserViewDelegate {
        fn on_popup_browser_view_created(
            &self,
            _browser_view: Option<&mut BrowserView>,
            popup_browser_view: Option<&mut BrowserView>,
            _is_devtools: i32,
        ) -> i32 {
            // Create a new top-level Window for the popup. It will show itself after
            // creation.
            let mut window_delegate = HelloWindowDelegate::new(
                RefCell::new(popup_browser_view.cloned()),
                self.runtime_style,
                ShowState::NORMAL,
            );
            window_create_top_level(Some(&mut window_delegate));

            // We created the Window.
            1
        }

        fn browser_runtime_style(&self) -> RuntimeStyle {
            self.runtime_style
        }
    }
}

wrap_app! {
    pub(crate) struct HelloApp;

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(HelloBrowserProcessHandler::new(RefCell::new(None)))
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
    struct HelloBrowserProcessHandler {
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
                // HelloHandler implements browser-level callbacks.
                let mut client = self.client.borrow_mut();
                *client = Some(HelloHandlerClient::new(HelloHandler::new(use_alloy_style)));
            }

            // Specify CEF browser settings here.
            let settings = BrowserSettings::default();

            let url = CefString::from(WebPage::Home.url().as_str());

            // Register the APP_SCHEME handler
            if let Err(error) = WebPageResourceHandlerFactory::<HelloWebPageHandler>::register(APP_SCHEME, None) {
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
                let mut delegate = HelloBrowserViewDelegate::new(runtime_style);
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
                let mut delegate = HelloWindowDelegate::new(
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
                let window_info = window_info.set_as_popup(Default::default(), "hello");

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
