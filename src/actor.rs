//! simple actor framework

use std::marker::PhantomData;

use futures_util::core_reexport::future::Future;
use tokio::sync::mpsc;
pub use tokio::task::JoinHandle;

/// address of actor
pub type Addr<T> = mpsc::Sender<T>;

/// Message box for actors
pub type MailBox<T> = mpsc::Receiver<T>;

/// actor
#[async_trait::async_trait]
pub trait Actor: Send + Sized + 'static {
    /// message type
    type Message: Send;

    type Context: ActorContext<Self>; // FUTURE: = Context<Self>;

    async fn on_message(&mut self, msg: Self::Message, ctx: &Self::Context);

    async fn setup(&mut self, _ctx: &Self::Context) {}
    async fn tear_down(&mut self, _ctx: &Self::Context) {}

    fn start(self) -> Addr<Self::Message> {
        Self::Context::run(self)
    }
}

/// Abstraction for async system
pub trait AsyncSystem: std::marker::Sync + std::marker::Send + 'static {
    fn spawn<T>(task: T) -> JoinHandle<T::Output>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static;
}

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

/// Actor context
pub trait ActorContext<A: Actor>: std::marker::Sync + std::marker::Send + 'static {
    fn addr(&self) -> Addr<A::Message>;

    fn spawn<F: Future>(f: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;

    fn run(actor: A) -> Addr<A::Message>;
}

pub type Context<A> = BasicActorContext<A, TokioSystem>;

/// Actor context
///
/// Used to get address or spawn actors
pub struct BasicActorContext<A: Actor, S: AsyncSystem> {
    tx: Addr<A::Message>,
    rx: MailBox<A::Message>,
    _async_system: PhantomData<S>,
}

impl<A, S> BasicActorContext<A, S>
where
    A: Actor<Context = Self>,
    S: AsyncSystem,
{
    fn new(tx: Addr<A::Message>, rx: MailBox<A::Message>) -> A::Context {
        Self {
            tx,
            rx,
            _async_system: PhantomData,
        }
    }

    async fn into_future(mut self: Self, mut actor: A) -> () {
        actor.setup(&self).await;
        while let Some(msg) = self.rx.recv().await {
            actor.on_message(msg, &self).await;
        }
        actor.tear_down(&self).await;
    }
}

impl<A: Actor, S: AsyncSystem> ActorContext<A> for BasicActorContext<A, S>
where
    A: Actor<Context = Self>,
    S: AsyncSystem,
{
    fn addr(&self) -> Addr<A::Message> {
        self.tx.clone()
    }

    fn spawn<F: Future>(f: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        S::spawn(f)
    }

    fn run(actor: A) -> Addr<A::Message> {
        let (tx, rx) = mpsc::channel(64); // TODO: config to change size
        let ctx = Self::new(tx.clone(), rx);
        S::spawn(ctx.into_future(actor));
        tx
    }
}
