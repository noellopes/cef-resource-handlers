use cef::*;
use std::sync::{Arc, Mutex, OnceLock, Weak};

fn get_data_uri(data: &[u8], mime_type: &str) -> String {
    let data = CefString::from(&base64_encode(Some(data)));
    let uri = CefString::from(&uriencode(Some(&data), 0)).to_string();
    format!("data:{mime_type};base64,{uri}")
}

#[cfg(target_os = "macos")]
mod mac;
#[cfg(target_os = "macos")]
use mac::*;

#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "windows")]
use win::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux::*;

static APP_HANDLER_INSTANCE: OnceLock<Weak<Mutex<AppHandler>>> = OnceLock::new();

pub struct AppHandler {
    is_alloy_style: bool,
    browser_list: Vec<Browser>,
    #[cfg(target_os = "macos")]
    is_closing: bool,
    #[cfg(target_os = "macos")]
    weak_self: Weak<Mutex<Self>>,
}

impl AppHandler {
    #[cfg(target_os = "macos")]
    pub fn instance() -> Option<Arc<Mutex<Self>>> {
        APP_HANDLER_INSTANCE.get().and_then(|weak| weak.upgrade())
    }

    pub fn new(is_alloy_style: bool) -> Arc<Mutex<Self>> {
        Arc::new_cyclic(|weak| {
            if let Err(instance) = APP_HANDLER_INSTANCE.set(weak.clone()) {
                assert_eq!(instance.strong_count(), 0, "Replacing a viable instance");
            }

            Mutex::new(Self {
                is_alloy_style,
                browser_list: Vec::new(),
                #[cfg(target_os = "macos")]
                is_closing: false,
                #[cfg(target_os = "macos")]
                weak_self: weak.clone(),
            })
        })
    }

    fn on_title_change(&mut self, browser: Option<&mut Browser>, title: Option<&CefString>) {
        debug_assert_ne!(currently_on(ThreadId::UI), 0);

        let mut browser = browser.cloned();
        if let Some(browser_view) = browser_view_get_for_browser(browser.as_mut()) {
            if let Some(window) = browser_view.window() {
                window.set_title(title);
            }
        } else if self.is_alloy_style {
            platform_title_change(browser.as_mut(), title);
        }
    }

    fn on_after_created(&mut self, browser: Option<&mut Browser>) {
        debug_assert_ne!(currently_on(ThreadId::UI), 0);

        let browser = browser.cloned().expect("Browser is None");

        assert_eq!(
            browser.host().expect("BrowserHost is None").runtime_style(),
            if self.is_alloy_style {
                RuntimeStyle::ALLOY
            } else {
                RuntimeStyle::CHROME
            }
        );

        self.browser_list.push(browser);
    }

    fn do_close(&mut self, _browser: Option<&mut Browser>) -> bool {
        debug_assert_ne!(currently_on(ThreadId::UI), 0);

        #[cfg(target_os = "macos")]
        if self.browser_list.len() == 1 {
            self.is_closing = true;
        }

        false
    }

    fn on_before_close(&mut self, browser: Option<&mut Browser>) {
        debug_assert_ne!(currently_on(ThreadId::UI), 0);

        let mut browser = browser.cloned().expect("Browser is None");
        if let Some(index) = self
            .browser_list
            .iter()
            .position(move |elem| elem.is_same(Some(&mut browser)) != 0)
        {
            self.browser_list.remove(index);
        }

        if self.browser_list.is_empty() {
            quit_message_loop();
        }
    }

    fn on_load_error(
        &mut self,
        _browser: Option<&mut Browser>,
        frame: Option<&mut Frame>,
        error_code: Errorcode,
        error_text: Option<&CefString>,
        failed_url: Option<&CefString>,
    ) {
        debug_assert_ne!(currently_on(ThreadId::UI), 0);

        if !self.is_alloy_style {
            return;
        }

        let error_code = sys::cef_errorcode_t::from(error_code);
        if error_code == sys::cef_errorcode_t::ERR_ABORTED {
            return;
        }
        let error_code = error_code as i32;

        let frame = frame.expect("Frame is None");
        let error_text = error_text.map(CefString::to_string).unwrap_or_default();
        let failed_url = failed_url.map(CefString::to_string).unwrap_or_default();
        let data = format!(
            r#"
            <html>
                <body bgcolor="white">
                    <h2>Failed to load URL {failed_url} with error {error_text} ({error_code}).</h2>
                </body>
            </html>
            "#
        );

        let uri = get_data_uri(data.as_bytes(), "text/html");
        let uri = CefString::from(uri.as_str());
        frame.load_url(Some(&uri));
    }

    #[cfg(target_os = "macos")]
    pub fn show_main_window(&mut self) {
        let thread_id = ThreadId::UI;
        if currently_on(thread_id) == 0 {
            let this = self
                .weak_self
                .upgrade()
                .expect("Weak reference to AppHandler is None");
            let mut task = ShowMainWindow::new(this);
            post_task(thread_id, Some(&mut task));
            return;
        }

        let Some(mut main_browser) = self.browser_list.first().cloned() else {
            return;
        };

        if let Some(browser_view) = browser_view_get_for_browser(Some(&mut main_browser)) {
            if let Some(window) = browser_view.window() {
                window.show();
            }
        } else if self.is_alloy_style {
            platform_show_window(Some(&mut main_browser));
        }
    }

    #[cfg(target_os = "macos")]
    pub fn close_all_browsers(&mut self, force_close: bool) {
        let thread_id = ThreadId::UI;
        if currently_on(thread_id) == 0 {
            let this = self
                .weak_self
                .upgrade()
                .expect("Weak reference to AppHandler is None");
            let mut task = CloseAllBrowsers::new(this, force_close);
            post_task(thread_id, Some(&mut task));
            return;
        }

        for browser in self.browser_list.iter() {
            let browser_host = browser.host().expect("BrowserHost is None");
            browser_host.close_browser(force_close.into());
        }
    }

    #[cfg(target_os = "macos")]
    pub fn is_closing(&self) -> bool {
        self.is_closing
    }
}

wrap_client! {
    pub struct AppHandlerClient {
        inner: Arc<Mutex<AppHandler>>,
    }

    impl Client {
        fn display_handler(&self) -> Option<DisplayHandler> {
            Some(AppHandlerDisplayHandler::new(self.inner.clone()))
        }

        fn life_span_handler(&self) -> Option<LifeSpanHandler> {
            Some(AppHandlerLifeSpanHandler::new(self.inner.clone()))
        }

        fn load_handler(&self) -> Option<LoadHandler> {
            Some(AppHandlerLoadHandler::new(self.inner.clone()))
        }
    }
}

wrap_display_handler! {
    struct AppHandlerDisplayHandler {
        inner: Arc<Mutex<AppHandler>>,
    }

    impl DisplayHandler {
        fn on_title_change(&self, browser: Option<&mut Browser>, title: Option<&CefString>) {
            let mut inner = self.inner.lock().expect("Failed to lock inner");
            inner.on_title_change(browser, title);
        }
    }
}

wrap_life_span_handler! {
    struct AppHandlerLifeSpanHandler {
        inner: Arc<Mutex<AppHandler>>,
    }

    impl LifeSpanHandler {
        fn on_after_created(&self, browser: Option<&mut Browser>) {
            let mut inner = self.inner.lock().expect("Failed to lock inner");
            inner.on_after_created(browser);
        }

        fn do_close(&self, browser: Option<&mut Browser>) -> i32 {
            let mut inner = self.inner.lock().expect("Failed to lock inner");
            inner.do_close(browser).into()
        }

        fn on_before_close(&self, browser: Option<&mut Browser>) {
            let mut inner = self.inner.lock().expect("Failed to lock inner");
            inner.on_before_close(browser);
        }
    }
}

wrap_load_handler! {
    struct AppHandlerLoadHandler {
        inner: Arc<Mutex<AppHandler>>,
    }

    impl LoadHandler {
        fn on_load_error(
            &self,
            browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            error_code: Errorcode,
            error_text: Option<&CefString>,
            failed_url: Option<&CefString>,
        ) {
            let mut inner = self.inner.lock().expect("Failed to lock inner");
            inner.on_load_error(browser, frame, error_code, error_text, failed_url);
        }
    }
}

#[cfg(target_os = "macos")]
wrap_task! {
    struct ShowMainWindow {
        inner: Arc<Mutex<AppHandler>>,
    }

    impl Task {
        fn execute(&self) {
            debug_assert_ne!(currently_on(ThreadId::UI), 0);

            let mut inner = self.inner.lock().expect("Failed to lock inner");
            inner.show_main_window();
        }
    }
}

#[cfg(target_os = "macos")]
wrap_task! {
    struct CloseAllBrowsers {
        inner: Arc<Mutex<AppHandler>>,
        force_close: bool,
    }

    impl Task {
        fn execute(&self) {
            debug_assert_ne!(currently_on(ThreadId::UI), 0);

            let mut inner = self.inner.lock().expect("Failed to lock inner");
            inner.close_all_browsers(self.force_close);
        }
    }
}