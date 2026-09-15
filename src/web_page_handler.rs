/// Represents a handler for web page requests.
pub trait WebPageHandler {
    /// Shared state available while creating this handler.
    ///
    /// Use `()` when the handler does not require shared state.
    type Context: crate::SharedContext;

    /// Creates a handler for a request.
    fn from_request(
        request_info: &crate::RequestInfo,
        context: &Self::Context,
    ) -> Result<Self, crate::ResourceHandlerError>
    where
        Self: Sized;

    /// Renders this page as HTML.
    fn render(&self) -> String;
}
