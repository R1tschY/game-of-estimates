use std::future::Future;
use std::panic::{RefUnwindSafe, UnwindSafe};

/// Abstraction for async system
pub trait AsyncSystem: Sync + Send + 'static {
    type JoinHandle<T: Send>: JoinHandle<T>;

    fn spawn<T>(task: T) -> Self::JoinHandle<T::Output>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static;
}

pub trait JoinHandle<T>:
    Future<Output = Result<T, Self::Error>> + Send + Sync + UnwindSafe + RefUnwindSafe + Unpin
{
    type Error;

    fn abort(&self);
    fn is_finished(&self) -> bool;
}
