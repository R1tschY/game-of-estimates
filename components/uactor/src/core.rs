use tokio::macros::support::Future;
use tokio::task::JoinHandle;

/// Abstraction for async system
pub trait AsyncSystem: std::marker::Sync + std::marker::Send + 'static {
    fn spawn<T>(task: T) -> JoinHandle<T::Output>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static;
}
