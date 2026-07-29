use crate::read_progress::ReadProgress;
use crate::shared_state::SharedState;
use crate::{ContentProvider, ResourceHandlerError};
use cef::*;
use std::{marker::PhantomData, os::raw::c_int};

wrap_resource_handler! {
    struct CustomResourceHandler<T: ContentProvider> {
        state: SharedState<ContentProviderState<T>>,
    }

    impl ResourceHandler {
        fn open(
            &self,
            request: Option<&mut Request>,
            handle_request: Option<&mut c_int>,
            _callback: Option<&mut Callback>,
        ) -> c_int {
            if let Some(handle_request) = handle_request {
                *handle_request = true as c_int;
            }

            let Some(request) = request else {
                return false as c_int; // Cancel request
            };

            if let Err(error) = self.open(request) {
                eprintln!("[ResourceHandler::open] {error}");
                return false as c_int; // Cancel request
            }

            true as c_int // Request handled successfully
        }

        fn read(
            &self,
            data_out: *mut u8,
            bytes_to_read: c_int,
            bytes_read: Option<&mut c_int>,
            _callback: Option<&mut ResourceReadCallback>,
        ) -> c_int {
            let bytes_to_read = bytes_to_read as usize;

            let result = match bytes_to_read {
                0 => Ok(0),
                _ => self.state.with_mut(|state| state.read(data_out, bytes_to_read)).flatten(),
            };

            match result {
                Ok(bytes_successfully_read) => {
                    if let Some(bytes_read) = bytes_read {
                        *bytes_read = bytes_successfully_read as c_int;
                    }

                    (bytes_successfully_read > 0) as c_int
                }
                Err(error) => {
                    eprintln!("[ResourceHandler::read] {error}");
                    -2 // ERR_FAILED
                }
            }
        }

        fn skip(
            &self,
            bytes_to_skip: i64,
            bytes_skipped: Option<&mut i64>,
            _callback: Option<&mut ResourceSkipCallback>,
        ) -> c_int {
            let bytes_to_skip = bytes_to_skip as usize;

            let result = match bytes_to_skip {
                0 => Ok(0),
                _ => self.state.with_mut(|state| state.progress.advance(bytes_to_skip)),
            };

            match result {
                Ok(bytes_advanced) => {
                    if let Some(bytes_skipped) = bytes_skipped {
                        *bytes_skipped = bytes_advanced as i64;
                    }

                    (bytes_advanced > 0) as c_int
                }
                Err(error) => {
                    eprintln!("[ResourceHandler::skip] {error}");
                    -2 // ERR_FAILED
                }
            }
        }

        fn response_headers(
            &self,
            response: Option<&mut Response>,
            response_length: Option<&mut i64>,
            _redirect_url: Option<&mut CefString>,
        ) {
            let Some(response) = response else {
                return;
            };

            let result = self.state.with(|state| state.response_headers(response, response_length));

            if let Err(error) = result {
                eprintln!("[ContentProvider::response_headers]: {error}");
                response.set_status(500); // Internal error
            }
        }
    }
}

impl<T: ContentProvider> CustomResourceHandler<T> {
    fn open(&self, request: &Request) -> Result<(), ResourceHandlerError> {
        let request_info = crate::RequestInfo::from_request(request)?;
        let content_provider = T::from_request(&request_info)?;
        self.state.set(ContentProviderState::new(content_provider))
    }
}

struct ContentProviderState<T: ContentProvider> {
    content_provider: T,
    progress: ReadProgress,
}

impl<T: ContentProvider> ContentProviderState<T> {
    fn new(content_provider: T) -> Self {
        let progress = ReadProgress::new(content_provider.size());

        Self {
            content_provider,
            progress,
        }
    }

    fn read(
        &mut self,
        data_out: *mut u8,
        max_bytes_to_read: usize,
    ) -> Result<usize, ResourceHandlerError> {
        let bytes_available = self.progress.bytes_available(max_bytes_to_read);

        if bytes_available == 0 {
            return Ok(0);
        }

        let content_provider = &mut self.content_provider;
        let bytes_read = content_provider.read(data_out, self.progress.offset, bytes_available)?;
        self.progress.offset += bytes_read;

        Ok(bytes_read)
    }

    fn response_headers(&self, response: &mut Response, response_length: Option<&mut i64>) {
        if let Some(response_length) = response_length {
            *response_length = self.progress.length.map_or(-1, |length| length as i64);
        }

        let mime_type = CefString::from(self.content_provider.mime_type());

        self.set_response_cache(response);
        response.set_mime_type(Some(&mime_type));
        response.set_status(200); // OK
    }

    fn set_response_cache(&self, response: &mut Response) {
        fn set_header_by_name(response: &mut Response, name: &str, value: &str) {
            response.set_header_by_name(
                Some(&CefString::from(name)),
                Some(&CefString::from(value)),
                true as c_int,
            );
        }

        if self.content_provider.should_cache() {
            set_header_by_name(response, "Cache-Control", "public,max-age=31536000"); // Cache for one year
        } else {
            for &(name, value) in NO_CACHE_HEADERS.iter() {
                set_header_by_name(response, name, value);
            }
        }
    }
}

const NO_CACHE_HEADERS: [(&str, &str); 3] = [
    ("Cache-Control", "no-cache,no-store,must-revalidate"),
    ("Pragma", "no-cache"),
    ("Expires", "0"),
];

wrap_scheme_handler_factory! {
    pub struct CustomResourceHandlerFactory<T: ContentProvider> {
        _phantom: PhantomData<T>,
    }

    impl SchemeHandlerFactory {
        fn create(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _scheme_name: Option<&CefString>,
            _request: Option<&mut Request>,
        ) -> Option<ResourceHandler> {
            Some(CustomResourceHandler::<T>::new(SharedState::new()))
        }
    }
}

impl<T: ContentProvider> CustomResourceHandlerFactory<T> {
    /// Registers the `CustomResourceHandlerFactory` for a given scheme and optional domain.
    pub fn register(
        scheme_name: &str,
        domain_name: Option<&str>,
    ) -> Result<(), ResourceHandlerError> {
        let scheme_name = CefString::from(scheme_name);
        let domain_name = domain_name.map(CefString::from);
        let mut factory = Self::new(PhantomData);

        if register_scheme_handler_factory(
            Some(&scheme_name),
            domain_name.as_ref(),
            Some(&mut factory),
        ) == false as c_int
        {
            let scheme_description = if let Some(domain) = domain_name {
                format!("'{scheme_name}' for domain '{domain}'")
            } else {
                format!("'{scheme_name}'")
            };

            Err(ResourceHandlerError::RegisterSchemeError(
                scheme_description,
            ))
        } else {
            Ok(())
        }
    }
}
