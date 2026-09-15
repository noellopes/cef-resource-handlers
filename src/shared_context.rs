/// Shared application context for page handlers and content providers.
///
/// Contexts must be clonable, thread-safe, and `'static` because they may be
/// shared across CEF callbacks. This trait is implemented automatically for
/// all `Clone + Send + Sync + 'static` types.
///
/// Use `Arc<Mutex<T>>` when the context needs shared mutable state.
pub trait SharedContext: Clone + Send + Sync + 'static {}

impl<T: Clone + Send + Sync + 'static> SharedContext for T {}
