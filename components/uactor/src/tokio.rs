use std::future::Future;

use tokio::task::JoinHandle;

use crate::core::AsyncSystem;

/// Tokio as async system
pub struct TokioSystem;

impl AsyncSystem for TokioSystem {
    fn spawn<T>(task: T) -> JoinHandle<<T as Future>::Output>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        tokio::spawn(task)
    }
}
