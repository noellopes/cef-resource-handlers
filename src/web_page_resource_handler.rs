use crate::{ContentProvider, CustomResourceHandlerFactory, WebPageHandler};
use std::marker::PhantomData;

/// A content provider that provides HTML content generate by a `WebPageHandler`.
pub struct WebPageContentProvider<T: WebPageHandler> {
    html_content: Vec<u8>,
    phantom: PhantomData<T>,
}

impl<T: WebPageHandler> ContentProvider for WebPageContentProvider<T> {
    fn from_request(
        request_info: &crate::RequestInfo,
    ) -> Result<Self, crate::ResourceHandlerError> {
        let handler = T::from_request(request_info)?;
        let html_content = handler.render().into_bytes();

        Ok(Self {
            html_content,
            phantom: PhantomData,
        })
    }

    fn size(&self) -> Option<usize> {
        Some(self.html_content.len())
    }

    fn mime_type(&self) -> &str {
        mime_guess::mime::TEXT_HTML_UTF_8.essence_str()
    }

    fn should_cache(&self) -> bool {
        false
    }

    fn read(
        &mut self,
        data_out: *mut u8,
        offset: usize,
        bytes_to_read: usize,
    ) -> Result<usize, crate::ResourceHandlerError> {
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.html_content.as_ptr().add(offset),
                data_out,
                bytes_to_read,
            );
        }
        Ok(bytes_to_read)
    }
}

/// A factory for creating `WebPageContentProvider` instances.
pub type WebPageResourceHandlerFactory<T> = CustomResourceHandlerFactory<WebPageContentProvider<T>>;
