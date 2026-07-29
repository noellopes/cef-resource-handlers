/// Represents a handler for web page requests. It provides methods for creating an instance from a request and rendering the content of the web page.
pub trait WebPageHandler {
    fn from_request(request_info: &crate::RequestInfo) -> Result<Self, crate::ResourceHandlerError>
    where
        Self: Sized;

    fn render(&self) -> String;
}
