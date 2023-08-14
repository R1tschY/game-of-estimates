use std::future::Future;

use tokio::task::{JoinError, JoinHandle as TokioJoinHandle};

use crate::core::{AsyncSystem, JoinHandle};

/// Tokio as async system
pub struct TokioSystem;

impl AsyncSystem for TokioSystem {
    type JoinHandle<T: Send> = TokioJoinHandle<T>;

    fn spawn<T>(task: T) -> TokioJoinHandle<<T as Future>::Output>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        tokio::spawn(task)
    }
}

impl<T: Send> JoinHandle<T> for TokioJoinHandle<T> {
    type Error = JoinError;

    fn abort(&self) {
        self.abort();
    }

    fn is_finished(&self) -> bool {
        self.is_finished()
    }
}

pub mod blocking {
    use super::*;
    use crate::blocking::*;

    pub type Context<A> = BasicActorContext<A, TokioSystem>;
}

pub mod nonblocking {
    use super::*;
    use crate::nonblocking::*;

    pub type Context<A> = BasicActorContext<A, TokioSystem>;
}
