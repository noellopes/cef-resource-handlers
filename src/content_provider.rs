/// Provides content and metadata for a resource handler.
/// A `ContentProvider` abstracts over different content sources such as local
/// files, network resources, or dynamically generated data, exposing both
/// metadata and byte-level access to the content.
pub trait ContentProvider {
    /// Creates a content provider for the given request.
    fn from_request(request_info: &crate::RequestInfo) -> Result<Self, crate::ResourceHandlerError>
    where
        Self: Sized;

    /// Returns the content size in bytes, or `None` if unknown.
    fn size(&self) -> Option<usize>;

    /// Returns the MIME type of the content.
    fn mime_type(&self) -> &str;

    /// Returns whether the content should be cached by the browser.
    fn should_cache(&self) -> bool;

    /// Reads a chunk of data starting at `offset` into `data_out`.
    /// Returns the number of bytes read, or an error on failure.
    /// May be called multiple times during content consumption.
    fn read(
        &mut self,
        data_out: *mut u8,
        offset: usize,
        bytes_to_read: usize,
    ) -> Result<usize, crate::ResourceHandlerError>;
}
